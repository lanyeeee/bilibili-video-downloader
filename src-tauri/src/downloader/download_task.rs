use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context};
use fs4::fs_std::FileExt;
use parking_lot::{Mutex, RwLock};
use tauri::AppHandle;
use tauri_specta::Event;
use tokio::{
    sync::{watch, SemaphorePermit},
    task::JoinSet,
    time::sleep,
};

use crate::{
    bili_client::BiliClient,
    danmaku_xml_to_ass::xml_to_ass,
    downloader::episode_type::EpisodeType,
    events::DownloadEvent,
    extensions::{AnyhowErrorToStringChain, AppHandleExt},
    types::{
        bangumi_info::BangumiInfo, cheese_info::CheeseInfo,
        create_download_task_params::CreateDownloadTaskParams,
        get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
        get_normal_info_params::GetNormalInfoParams, normal_info::NormalInfo,
    },
    utils::{self, ToXml},
};

use super::{download_progress::DownloadProgress, download_task_state::DownloadTaskState};

pub struct DownloadTask {
    pub app: AppHandle,
    pub state_sender: watch::Sender<DownloadTaskState>,
    pub restart_sender: watch::Sender<()>,
    pub cancel_sender: watch::Sender<()>,
    pub delete_sender: watch::Sender<()>,
    pub task_id: String,
    pub progress: RwLock<DownloadProgress>,
}

impl DownloadTask {
    pub fn from_params(app: &AppHandle, params: &CreateDownloadTaskParams) -> Vec<Arc<Self>> {
        use CreateDownloadTaskParams::{Bangumi, Cheese, Normal};

        let mut progresses = Vec::new();
        match params {
            Normal(params) => {
                for &(aid, cid) in &params.aid_cid_pairs {
                    let progress = match DownloadProgress::from_normal(app, &params.info, aid, cid)
                    {
                        Ok(progress) => progress,
                        Err(err) => {
                            let cid = cid.map_or("None".to_string(), |id| id.to_string());
                            let ids_string = format!("aid: {aid}, cid: {cid}");
                            let err_title = format!("{ids_string} 创建普通视频的下载进度失败");
                            let string_chain = err.to_string_chain();
                            tracing::error!(err_title, message = string_chain);
                            continue;
                        }
                    };

                    progresses.extend(progress);
                }
            }
            Bangumi(params) => {
                for ep_id in &params.ep_ids {
                    let progress = match DownloadProgress::from_bangumi(app, &params.info, *ep_id) {
                        Ok(progress) => progress,
                        Err(err) => {
                            let ids_string = format!("ep_id: {ep_id}");
                            let err_title = format!("{ids_string} 创建番剧的下载进度失败");
                            let string_chain = err.to_string_chain();
                            tracing::error!(err_title, message = string_chain);
                            continue;
                        }
                    };

                    progresses.push(progress);
                }
            }
            Cheese(params) => {
                for ep_id in &params.ep_ids {
                    let progress = match DownloadProgress::from_cheese(app, &params.info, *ep_id) {
                        Ok(progress) => progress,
                        Err(err) => {
                            let ids_string = format!("ep_id: {ep_id}");
                            let err_title = format!("{ids_string} 创建课程的下载进度失败");
                            let string_chain = err.to_string_chain();
                            tracing::error!(err_title, message = string_chain);
                            continue;
                        }
                    };

                    progresses.push(progress);
                }
            }
        }

        let mut tasks = Vec::new();
        for progress in progresses {
            if let Err(err) = progress.save(app, true) {
                let ids_string = progress.get_ids_string();
                let episode_title = &progress.episode_title;
                let err_title = format!("{ids_string} `{episode_title}`保存下载进度到文件失败");
                let string_chain = err.to_string_chain();
                tracing::error!(err_title, message = string_chain);
            }

            let (state_sender, _) = watch::channel(DownloadTaskState::Pending);
            let (restart_sender, _) = watch::channel(());
            let (cancel_sender, _) = watch::channel(());
            let (delete_sender, _) = watch::channel(());

            let task = Arc::new(Self {
                app: app.clone(),
                state_sender,
                restart_sender,
                cancel_sender,
                delete_sender,
                task_id: progress.task_id.clone(),
                progress: RwLock::new(progress),
            });

            tauri::async_runtime::spawn(task.clone().process());

            tasks.push(task);
        }

        tasks
    }

    pub fn from_progress(app: AppHandle, progress: DownloadProgress) -> Arc<Self> {
        let init_state = if progress.is_completed() {
            DownloadTaskState::Completed
        } else {
            DownloadTaskState::Paused
        };
        let (state_sender, _) = watch::channel(init_state);
        let (restart_sender, _) = watch::channel(());
        let (cancel_sender, _) = watch::channel(());
        let (delete_sender, _) = watch::channel(());

        let task = Arc::new(Self {
            app,
            state_sender,
            restart_sender,
            cancel_sender,
            delete_sender,
            task_id: progress.task_id.clone(),
            progress: RwLock::new(progress),
        });

        tauri::async_runtime::spawn(task.clone().process());

        task
    }

    async fn process(self: Arc<Self>) {
        let task_id = &self.task_id;
        let state = *self.state_sender.borrow();
        let progress = self.progress.read().clone();
        let _ = DownloadEvent::TaskCreate { state, progress }.emit(&self.app);

        let mut state_receiver = self.state_sender.subscribe();
        state_receiver.mark_changed();

        let mut restart_receiver = self.restart_sender.subscribe();
        let mut cancel_receiver = self.cancel_sender.subscribe();
        let mut delete_receiver = self.delete_sender.subscribe();

        let mut permit = None;
        let mut download_task_option = None;

        loop {
            let state = *state_receiver.borrow();
            let state_is_downloading = state == DownloadTaskState::Downloading;
            let state_is_pending = state == DownloadTaskState::Pending;

            let download_task = async {
                download_task_option
                    .get_or_insert(Box::pin(self.download()))
                    .await;
            };

            tokio::select! {
                () = download_task, if state_is_downloading && permit.is_some() => {
                    download_task_option = None;
                    if let Some(permit) = permit.take() {
                        drop(permit);
                    };
                }

                () = self.acquire_task_permit(&mut permit), if state_is_pending => {},

                _ = state_receiver.changed() => {
                    self.handle_state_change(&mut permit, &mut state_receiver).await;
                }

                _ = restart_receiver.changed() => {
                    self.handle_restart_notify();
                    tracing::debug!("ID为`{task_id}`的下载任务已重来");
                    download_task_option = None;
                }

                _ = cancel_receiver.changed() => return,

                _ = delete_receiver.changed() => {
                    let _ = DownloadEvent::TaskDelete {
                        task_id: self.task_id.clone(),
                    }
                    .emit(&self.app);

                    if permit.is_some() {
                        // 如果有permit则稍微等一下再退出
                        // 这是为了避免大批量删除时，本应删除的任务因拿到permit而又稍微下载一小段
                        sleep(Duration::from_millis(100)).await;
                    }

                    tracing::debug!("ID为`{task_id}`的下载任务已删除");
                    return;
                }
            }
        }
    }

    async fn download(self: &Arc<Self>) {
        let mut progress = self.progress.read().clone();
        let ids_string = progress.get_ids_string();
        let episode_title = progress.episode_title.clone();

        if progress.is_completed() {
            tracing::info!("{ids_string} 跳过`{episode_title}`的下载，因为它已经完成");
            self.set_state(DownloadTaskState::Completed);
            return;
        }

        tracing::debug!("{ids_string} 开始准备`{episode_title}`的下载");
        let _ = DownloadEvent::ProgressPreparing {
            task_id: self.task_id.clone(),
        }
        .emit(&self.app);

        if let Err(err) = progress.prepare(&self.app).await {
            let err_title = format!("{ids_string} `{episode_title}`准备下载失败");
            let string_chain = err.to_string_chain();
            tracing::error!(err_title, message = string_chain);

            self.set_state(DownloadTaskState::Failed);

            return;
        }

        progress.completed_ts = None; // 重置完成时间戳
        self.update_progress(|p| *p = progress.clone());

        tracing::debug!("{ids_string} 开始下载`{episode_title}`");
        if let Err(err) = self
            .handle_progress(progress)
            .await
            .context("[继续]失败的任务可以断点续传")
        {
            let err_title = format!("{ids_string} `{episode_title}`下载失败");
            let string_chain = err.to_string_chain();
            tracing::error!(err_title, message = string_chain);

            self.set_state(DownloadTaskState::Failed);

            return;
        }

        self.sleep_between_task().await;

        self.set_state(DownloadTaskState::Completed);
        tracing::info!("{ids_string} `{episode_title}`下载完成");
    }

    async fn handle_progress(self: &Arc<Self>, progress: DownloadProgress) -> anyhow::Result<()> {
        let ids_string = progress.get_ids_string();
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        std::fs::create_dir_all(episode_dir).context(format!(
            "{ids_string} 创建目录`{}`失败",
            episode_dir.display()
        ))?;

        if !progress.video_task.is_completed() && progress.video_task.content_length != 0 {
            // 如果视频任务被选中且未完成且有要下载的内容，则下载视频
            self.download_video(&progress)
                .await
                .context(format!("{ids_string} `{filename}`下载视频文件失败"))?;
            tracing::debug!("{ids_string} `{filename}`视频下载完成");
        }

        if !progress.audio_task.is_completed() && progress.audio_task.content_length != 0 {
            // 如果音频任务被选中且未完成且有要下载的内容，则下载音频
            self.download_audio(&progress)
                .await
                .context(format!("{ids_string} `{filename}`下载音频文件失败"))?;
            tracing::debug!("{ids_string} `{filename}`音频下载完成");
        }

        if !progress.merge_task.is_completed() {
            self.merge_video_audio(&progress)
                .await
                .context(format!("{ids_string} `{filename}`合并视频和音频失败"))?;
            tracing::debug!("{ids_string} `{filename}`视频和音频合并完成");
        }

        if !progress.danmaku_task.is_completed() {
            self.download_danmaku(&progress)
                .await
                .context(format!("{ids_string} `{filename}`下载弹幕失败"))?;
            tracing::debug!("{ids_string} `{filename}`弹幕下载完成");
        }

        if !progress.subtitle_task.is_completed() {
            self.download_subtitle(&progress)
                .await
                .context(format!("{ids_string} `{filename}`下载字幕失败"))?;
            tracing::debug!("{ids_string} `{filename}`字幕下载完成");
        }

        if !progress.cover_task.is_completed() {
            self.download_cover(&progress)
                .await
                .context(format!("{ids_string} `{filename}`下载封面失败"))?;
            tracing::debug!("{ids_string} `{filename}`封面下载完成");
        }

        let mut episode_info = None;

        if !progress.nfo_task.is_completed() {
            self.download_nfo(&progress, &mut episode_info)
                .await
                .context(format!("{ids_string} `{filename}`下载NFO失败"))?;
            tracing::debug!("{ids_string} `{filename}`NFO下载完成");
        }

        if !progress.json_task.is_completed() {
            self.download_json(&progress, &mut episode_info)
                .await
                .context(format!("{ids_string} `{filename}`下载JSON元数据失败"))?;
            tracing::debug!("{ids_string} `{filename}`JSON元数据下载完成");
        }

        let completed_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();
        if completed_ts.is_some() {
            self.update_progress(|p| p.completed_ts = completed_ts);
        }

        Ok(())
    }

    async fn download_video(self: &Arc<Self>, progress: &DownloadProgress) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let temp_file_path = episode_dir.join(format!(
            "{filename}.mp4.com.lanyeeee.bilibili-video-downloader"
        ));

        let (video_task, episode_title, ids_string) = {
            let progress = self.progress.read();
            (
                progress.video_task.clone(),
                progress.episode_title.clone(),
                progress.get_ids_string(),
            )
        };

        let file = if temp_file_path.exists() {
            // 如果临时文件已存在，则打开它
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&temp_file_path)?
        } else {
            // 如果临时文件不存在，创建它并预分配空间
            let file = File::create(&temp_file_path)?;
            file.allocate(video_task.content_length)?;
            file
        };
        let file = Arc::new(Mutex::new(file));

        let chunk_count = video_task.chunks.len();

        let mut join_set = JoinSet::new();
        for (i, chunk) in video_task.chunks.iter().enumerate() {
            if chunk.completed {
                continue;
            }

            let (start, end) = (chunk.start, chunk.end);

            let download_chunk_task = DownloadChunkTask {
                download_task: self.clone(),
                start,
                end,
                url: video_task.url.to_string(),
                file: file.clone(),
                chunk_index: i,
            };

            let chunk_order = i + 1;

            join_set.spawn(async move {
                download_chunk_task.process().await.context(format!(
                    "分片`{chunk_order}/{chunk_count}`下载失败({start}-{end})"
                ))
            });
        }

        while let Some(Ok(download_video_result)) = join_set.join_next().await {
            match download_video_result {
                Ok(i) => self.update_progress(|p| p.video_task.chunks[i].completed = true),
                Err(err) => {
                    let err_title = format!("{ids_string} `{episode_title}`视频的一个分片下载失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                }
            }
        }
        // 检查视频是否已下载完成
        let download_completed = self
            .progress
            .read()
            .video_task
            .chunks
            .iter()
            .all(|chunk| chunk.completed);
        if !download_completed {
            return Err(anyhow!(
                "视频文件`{}`有分片未下载完成，[继续]可以跳过已下载分片断点续传",
                temp_file_path.display()
            ));
        }

        let is_video_file_complete = utils::is_mp4_complete(&temp_file_path).context(format!(
            "检查视频文件`{}`是否完整失败",
            temp_file_path.display()
        ))?;

        if !is_video_file_complete {
            self.update_progress(|p| p.video_task.mark_uncompleted());
            return Err(anyhow!(
                "视频文件`{}`不完整，[继续]会重新下载所有分片",
                temp_file_path.display()
            ));
        }

        // 重命名临时文件
        let mp4_path = episode_dir.join(format!("{filename}.mp4"));
        if mp4_path.exists() {
            std::fs::remove_file(&mp4_path)
                .context(format!("删除已存在的视频文件`{}`失败", mp4_path.display()))?;
        }
        std::fs::rename(&temp_file_path, &mp4_path).context(format!(
            "将临时文件`{}`重命名为`{}`失败",
            temp_file_path.display(),
            mp4_path.display()
        ))?;

        self.update_progress(|p| p.video_task.completed = true);

        Ok(())
    }

    async fn download_audio(self: &Arc<Self>, progress: &DownloadProgress) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let temp_file_path = episode_dir.join(format!(
            "{filename}.m4a.com.lanyeeee.bilibili-video-downloader"
        ));
        let (audio_task, episode_title, ids_string) = {
            let progress = self.progress.read();
            (
                progress.audio_task.clone(),
                progress.episode_title.clone(),
                progress.get_ids_string(),
            )
        };

        let file = if temp_file_path.exists() {
            // 如果文件已存在，则打开它
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&temp_file_path)?
        } else {
            // 如果文件不存在，创建它并预分配空间
            let file = File::create(&temp_file_path)?;
            file.allocate(audio_task.content_length)?;
            file
        };
        let file = Arc::new(Mutex::new(file));

        let chunk_count = audio_task.chunks.len();

        let mut join_set = JoinSet::new();
        for (chunk_index, chunk) in audio_task.chunks.iter().enumerate() {
            if chunk.completed {
                continue;
            }

            let (start, end) = (chunk.start, chunk.end);

            let download_chunk_task = DownloadChunkTask {
                download_task: self.clone(),
                start,
                end,
                url: audio_task.url.to_string(),
                file: file.clone(),
                chunk_index,
            };

            join_set.spawn(async move {
                download_chunk_task.process().await.context(format!(
                    "分片`{chunk_index}/{chunk_count}`下载失败({start}-{end})"
                ))
            });
        }

        while let Some(Ok(download_video_result)) = join_set.join_next().await {
            match download_video_result {
                Ok(i) => self.update_progress(|p| p.audio_task.chunks[i].completed = true),
                Err(err) => {
                    let err_title = format!("{ids_string} `{episode_title}`音频的一个分片下载失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                }
            }
        }

        let download_completed = self
            .progress
            .read()
            .audio_task
            .chunks
            .iter()
            .all(|chunk| chunk.completed);
        if !download_completed {
            return Err(anyhow!(
                "音频文件`{}`有分片未下载完成，[继续]可以跳过已下载分片断点续传",
                temp_file_path.display()
            ));
        }

        let is_audio_file_complete = utils::is_mp4_complete(&temp_file_path).context(format!(
            "检查音频文件`{}`是否完整失败",
            temp_file_path.display()
        ))?;

        if !is_audio_file_complete {
            self.update_progress(|p| p.video_task.mark_uncompleted());
            return Err(anyhow!(
                "音频文件`{}`不完整，[继续]会重新下载所有分片",
                temp_file_path.display()
            ));
        }

        // 重命名临时文件
        let m4a_path = episode_dir.join(format!("{filename}.m4a"));
        if m4a_path.exists() {
            std::fs::remove_file(&m4a_path)
                .context(format!("删除已存在的音频文件`{}`失败", m4a_path.display()))?;
        }
        std::fs::rename(&temp_file_path, &m4a_path).context(format!(
            "将临时文件`{}`重命名为`{}`失败",
            temp_file_path.display(),
            m4a_path.display()
        ))?;

        self.update_progress(|p| p.audio_task.completed = true);

        Ok(())
    }

    async fn merge_video_audio(&self, progress: &DownloadProgress) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            self.update_progress(|p| p.merge_task.completed = true);
            return Ok(());
        }

        let audio_path = episode_dir.join(format!("{filename}.m4a"));
        if !audio_path.exists() {
            self.update_progress(|p| p.merge_task.completed = true);
            return Ok(());
        }

        let output_path = episode_dir.join(format!("{filename}-merged.mp4"));

        let ffmpeg_program = std::env::current_exe()
            .context("获取当前可执行文件路径失败")?
            .parent()
            .context("获取当前可执行文件所在目录失败")?
            .join("com.lanyeeee.bilibili-video-downloader-ffmpeg");

        let (tx, rx) = tokio::sync::oneshot::channel();
        let video_path_clone = video_path.clone();
        let audio_path_clone = audio_path.clone();
        let output_path_clone = output_path.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut command = std::process::Command::new(ffmpeg_program);

            command
                .arg("-i")
                .arg(video_path_clone)
                .arg("-i")
                .arg(audio_path_clone)
                .arg("-c")
                .arg("copy")
                .arg("-map")
                .arg("0:v:0")
                .arg("-map")
                .arg("1:a:0")
                .arg(output_path_clone)
                .arg("-y");

            #[cfg(target_os = "windows")]
            {
                // 隐藏窗口
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x0800_0000);
            }

            let output = command.output();

            let _ = tx.send(output);
        });

        let output = rx.await??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let err = anyhow!(format!("STDOUT: {stdout}"))
                .context(format!("STDERR: {stderr}"))
                .context("原因可能是视频或音频文件损坏，建议[重来]试试");
            return Err(err);
        }

        std::fs::remove_file(&video_path)
            .context(format!("删除视频文件`{}`失败", video_path.display()))?;
        std::fs::remove_file(&audio_path)
            .context(format!("删除音频文件`{}`失败", audio_path.display()))?;
        std::fs::rename(&output_path, &video_path).context(format!(
            "将`{}`重命名为`{}`失败",
            output_path.display(),
            video_path.display()
        ))?;

        self.update_progress(|p| p.merge_task.completed = true);

        Ok(())
    }

    async fn download_danmaku(&self, progress: &DownloadProgress) -> anyhow::Result<()> {
        let (aid, cid, duration) = (progress.aid, progress.cid, progress.duration);
        let danmaku_task = &progress.danmaku_task;
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let bili_client = self.app.get_bili_client();
        let replies = bili_client
            .get_danmaku(aid, cid, duration)
            .await
            .context("获取弹幕失败")?;

        let xml = replies.to_xml(cid).context("将弹幕转换为XML失败")?;

        if danmaku_task.xml_selected {
            let xml_path = episode_dir.join(format!("{filename}.弹幕.xml"));
            std::fs::write(&xml_path, &xml)
                .context(format!("保存弹幕XML到`{}`失败", xml_path.display()))?;
        }

        if danmaku_task.ass_selected {
            let config = self.app.get_config().read().danmaku_config.clone();
            let ass_path = episode_dir.join(format!("{filename}.弹幕.ass"));
            let ass_file = File::create(&ass_path)
                .context(format!("创建弹幕ASS文件`{}`失败", ass_path.display()))?;
            let title = filename.to_string();
            xml_to_ass(&xml, ass_file, title, config).context("将弹幕XML转换为ASS失败")?;
        }

        if danmaku_task.json_selected {
            let json_path = episode_dir.join(format!("{filename}.弹幕.json"));
            let json_string = serde_json::to_string(&replies).context("将弹幕转换为JSON失败")?;
            std::fs::write(&json_path, json_string)
                .context(format!("保存弹幕JSON到`{}`失败", json_path.display()))?;
        }

        self.update_progress(|p| p.danmaku_task.completed = true);

        Ok(())
    }

    async fn download_subtitle(&self, progress: &DownloadProgress) -> anyhow::Result<()> {
        use std::fmt::Write;

        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let (aid, cid) = {
            let progress = self.progress.read();
            (progress.aid, progress.cid)
        };

        let bili_client = self.app.get_bili_client();
        let player_info = bili_client
            .get_player_info(aid, cid)
            .await
            .context("获取播放器信息失败")?;

        let subtitle = &player_info.subtitle;
        for subtitle_detail in &subtitle.subtitles {
            let url = format!("http:{}", subtitle_detail.subtitle_url);
            let subtitle = bili_client
                .get_subtitle(&url)
                .await
                .context("获取字幕失败")?;

            let mut srt_content = String::new();
            for (i, b) in subtitle.body.iter().enumerate() {
                let index = i + 1;
                let content = &b.content;
                let start_time = utils::seconds_to_srt_time(b.from);
                let end_time = utils::seconds_to_srt_time(b.to);
                let _ = writeln!(
                    &mut srt_content,
                    "{index}\n{start_time} --> {end_time}\n{content}\n"
                );
            }

            let lan = utils::filename_filter(&subtitle_detail.lan);
            let save_path = episode_dir.join(format!("{filename}.{lan}.srt"));
            std::fs::write(save_path, srt_content)?;
        }

        self.update_progress(|p| p.subtitle_task.completed = true);

        Ok(())
    }

    async fn download_cover(&self, progress: &DownloadProgress) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let bili_client = self.app.get_bili_client();
        let (cover_data, ext) = bili_client
            .get_cover_data_and_ext(&progress.cover_task.url)
            .await
            .context("获取封面失败")?;

        let save_path = episode_dir.join(format!("{filename}.{ext}"));
        std::fs::write(&save_path, cover_data)
            .context(format!("保存封面到`{}`失败", save_path.display()))?;

        self.update_progress(|p| p.cover_task.completed = true);

        Ok(())
    }

    async fn download_nfo(
        &self,
        progress: &DownloadProgress,
        episode_info: &mut Option<EpisodeInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);
        let (aid, ep_id, episode_type) = (progress.aid, progress.ep_id, progress.episode_type);

        let bili_client = self.app.get_bili_client();

        let episode_info = episode_info
            .get_or_init(&bili_client, aid, ep_id, episode_type)
            .await?;

        match episode_info {
            EpisodeInfo::Normal(info) => {
                let tags = bili_client
                    .get_tags(aid)
                    .await
                    .context("获取视频标签失败")?;
                let movie_nfo = info
                    .to_movie_nfo(tags)
                    .context("将普通视频信息转换为movie NFO失败")?;
                let nfo_path = episode_dir.join(format!("{filename}.nfo"));
                std::fs::write(&nfo_path, movie_nfo)
                    .context(format!("保存普通视频NFO到`{}`失败", nfo_path.display()))?;

                if let Some(ugc_season) = &info.ugc_season {
                    let collection_cover = &ugc_season.cover;
                    let (cover_data, ext) = bili_client
                        .get_cover_data_and_ext(collection_cover)
                        .await
                        .context("获取普通视频合集封面失败")?;
                    let cover_path = episode_dir.join(format!("poster.{ext}"));
                    std::fs::write(&cover_path, cover_data).context(format!(
                        "保存普通视频合集封面到`{}`失败",
                        cover_path.display()
                    ))?;
                }
            }
            EpisodeInfo::Bangumi(info, ep_id) => {
                let tvshow_nfo = info
                    .to_tvshow_nfo()
                    .context("将番剧信息转换为tvshow NFO失败")?;
                let tvshow_nfo_path = episode_dir.join("tvshow.nfo");
                std::fs::write(&tvshow_nfo_path, tvshow_nfo)
                    .context(format!("保存番剧NFO到`{}`失败", tvshow_nfo_path.display()))?;

                let episode_details_nfo = info
                    .to_episode_details_nfo(*ep_id)
                    .context("将番剧信息转换为episodedetail NFO失败")?;
                let episode_details_nfo_path = episode_dir.join(format!("{filename}.nfo"));
                std::fs::write(&episode_details_nfo_path, episode_details_nfo).context(format!(
                    "保存番剧NFO到`{}`失败",
                    episode_details_nfo_path.display()
                ))?;

                let poster_url = &info.cover;
                let (poster_data, ext) = bili_client
                    .get_cover_data_and_ext(poster_url)
                    .await
                    .context("获取番剧封面失败")?;
                let poster_path = episode_dir.join(format!("poster.{ext}"));
                std::fs::write(&poster_path, poster_data)
                    .context(format!("保存番剧封面到`{}`失败", poster_path.display()))?;

                let fanart_url = &info.bkg_cover;
                if !fanart_url.is_empty() {
                    let (fanart_data, ext) = bili_client
                        .get_cover_data_and_ext(fanart_url)
                        .await
                        .context("获取番剧封面失败")?;
                    let fanart_path = episode_dir.join(format!("fanart.{ext}"));
                    std::fs::write(&fanart_path, fanart_data)
                        .context(format!("保存番剧封面到`{}`失败", fanart_path.display()))?;
                }
            }
            EpisodeInfo::Cheese(info, ep_id) => {
                let tvshow_nfo = info
                    .to_tvshow_nfo()
                    .context("将课程信息转换为tvshow NFO失败")?;
                let tvshow_nfo_path = episode_dir.join("tvshow.nfo");
                std::fs::write(&tvshow_nfo_path, tvshow_nfo)
                    .context(format!("保存课程NFO到`{}`失败", tvshow_nfo_path.display()))?;

                let episode_details_nfo = info
                    .to_episode_details_nfo(*ep_id)
                    .context("将课程信息转换为episodedetail NFO失败")?;
                let episode_details_nfo_path = episode_dir.join(format!("{filename}.nfo"));
                std::fs::write(&episode_details_nfo_path, episode_details_nfo).context(format!(
                    "保存课程NFO到`{}`失败",
                    episode_details_nfo_path.display()
                ))?;

                let poster_url = &info.cover;
                let (poster_data, ext) = bili_client
                    .get_cover_data_and_ext(poster_url)
                    .await
                    .context("获取课程封面失败")?;
                let poster_path = episode_dir.join(format!("poster.{ext}"));
                std::fs::write(&poster_path, poster_data)
                    .context(format!("保存课程封面到`{}`失败", poster_path.display()))?;
            }
        }

        self.update_progress(|p| p.nfo_task.completed = true);

        Ok(())
    }

    async fn download_json(
        &self,
        progress: &DownloadProgress,
        episode_info: &mut Option<EpisodeInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);
        let (aid, ep_id, episode_type) = (progress.aid, progress.ep_id, progress.episode_type);

        let bili_client = self.app.get_bili_client();

        let episode_info = episode_info
            .get_or_init(&bili_client, aid, ep_id, episode_type)
            .await?;

        let json_path = episode_dir.join(format!("{filename}-元数据.json"));
        let json_string = match episode_info {
            EpisodeInfo::Normal(info) => {
                serde_json::to_string(&info).context("将普通视频信息转换为JSON失败")?
            }
            EpisodeInfo::Bangumi(info, _ep_id) => {
                serde_json::to_string(&info).context("将番剧信息转换为JSON失败")?
            }
            EpisodeInfo::Cheese(info, _ep_id) => {
                serde_json::to_string(&info).context("将课程信息转换为JSON失败")?
            }
        };
        std::fs::write(&json_path, json_string)
            .context(format!("保存JSON到`{}`失败", json_path.display()))?;

        self.update_progress(|p| p.json_task.completed = true);

        Ok(())
    }

    async fn sleep_between_task(&self) {
        let task_id = &self.task_id;
        let mut remaining_sec = self.app.get_config().read().task_download_interval_sec;
        while remaining_sec > 0 {
            // 发送章节休眠事件
            let _ = DownloadEvent::TaskSleeping {
                task_id: task_id.clone(),
                remaining_sec,
            }
            .emit(&self.app);
            sleep(Duration::from_secs(1)).await;
            remaining_sec -= 1;
        }
    }

    async fn acquire_task_permit<'a>(&'a self, permit: &mut Option<SemaphorePermit<'a>>) {
        let (episode_title, ids_string) = {
            let progress = self.progress.read();
            (progress.episode_title.clone(), progress.get_ids_string())
        };

        *permit = match permit.take() {
            // 如果有permit，则直接用
            Some(permit) => Some(permit),
            // 如果没有permit，则获取permit
            None => match self
                .app
                .get_download_manager()
                .inner()
                .task_sem
                .acquire()
                .await
                .map_err(anyhow::Error::from)
            {
                Ok(permit) => Some(permit),
                Err(err) => {
                    let err_title =
                        format!("{ids_string} `{episode_title}`获取下载任务的permit失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);

                    self.set_state(DownloadTaskState::Failed);

                    return;
                }
            },
        };
        // 如果当前任务状态不是`Pending`，则不将任务状态设置为`Downloading`
        if *self.state_sender.borrow() != DownloadTaskState::Pending {
            return;
        }
        // 将任务状态设置为`Downloading`
        if let Err(err) = self
            .state_sender
            .send(DownloadTaskState::Downloading)
            .map_err(anyhow::Error::from)
        {
            let err_title = format!("{ids_string} `{episode_title}`发送状态`Downloading`失败");
            let string_chain = err.to_string_chain();
            tracing::error!(err_title, message = string_chain);

            self.set_state(DownloadTaskState::Failed);
        }
    }

    async fn handle_state_change<'a>(
        &'a self,
        permit: &mut Option<SemaphorePermit<'a>>,
        state_receiver: &mut watch::Receiver<DownloadTaskState>,
    ) {
        let state = *state_receiver.borrow();
        let task_id = self.task_id.clone();
        let _ = DownloadEvent::TaskStateUpdate { task_id, state }.emit(&self.app);

        if state == DownloadTaskState::Paused {
            // 稍微等一下再释放permit
            // 避免大批量暂停时，本应暂停的任务因拿到permit而稍微下载一小段(虽然最终会被暂停)
            sleep(Duration::from_millis(100)).await;
            let task_id = &self.task_id;
            tracing::debug!("ID为`{task_id}`的下载任务已暂停");
            if let Some(permit) = permit.take() {
                drop(permit);
            };
        }
    }

    fn handle_restart_notify(&self) {
        self.update_progress(|p| {
            p.mark_uncompleted();
        });
        self.set_state(DownloadTaskState::Pending);
    }

    pub fn set_state(&self, state: DownloadTaskState) {
        let (episode_title, ids_string) = {
            let progress = self.progress.read();
            (progress.episode_title.clone(), progress.get_ids_string())
        };

        if let Err(err) = self.state_sender.send(state).map_err(anyhow::Error::from) {
            let err_title = format!("{ids_string} `{episode_title}`发送状态`{state:?}`失败");
            let string_chain = err.to_string_chain();
            tracing::error!(err_title, message = string_chain);
        }
    }

    fn update_progress(&self, update_fn: impl FnOnce(&mut DownloadProgress)) {
        // 修改数据
        let updated_progress = {
            let mut progress = self.progress.write();
            update_fn(&mut progress);
            progress
        };
        // 发送更新事件并保存到文件
        let _ = DownloadEvent::ProgressUpdate {
            progress: updated_progress.clone(),
        }
        .emit(&self.app);

        if let Err(err) = updated_progress.save(&self.app, false) {
            let ids_string = updated_progress.get_ids_string();
            let episode_title = &updated_progress.episode_title;
            let err_title = format!("{ids_string} `{episode_title}`保存下载进度到文件失败");
            let string_chain = err.to_string_chain();
            tracing::error!(err_title, message = string_chain);
        }
    }
}

struct DownloadChunkTask {
    download_task: Arc<DownloadTask>,
    start: u64,
    end: u64,
    url: String,
    file: Arc<Mutex<File>>,
    chunk_index: usize,
}

impl DownloadChunkTask {
    async fn process(self) -> anyhow::Result<usize> {
        let download_chunk_task = self.download_chunk();
        tokio::pin!(download_chunk_task);

        let mut state_receiver = self.download_task.state_sender.subscribe();
        state_receiver.mark_changed();

        let mut restart_receiver = self.download_task.restart_sender.subscribe();
        let mut delete_receiver = self.download_task.delete_sender.subscribe();

        let mut permit = None;

        loop {
            let state_is_downloading = *state_receiver.borrow() == DownloadTaskState::Downloading;
            tokio::select! {
                result = &mut download_chunk_task, if state_is_downloading && permit.is_some() => break result,

                result = self.acquire_chunk_permit(&mut permit), if state_is_downloading && permit.is_none() => {
                    match result {
                        Ok(()) => {},
                        Err(err) => break Err(err),
                    }
                },

                _ = state_receiver.changed() => {
                    if *state_receiver.borrow() == DownloadTaskState::Paused {
                        // 稍微等一下再释放permit
                        sleep(Duration::from_millis(100)).await;
                        if let Some(permit) = permit.take() {
                            drop(permit);
                        };
                    }
                },

                _ = restart_receiver.changed() => break Ok(self.chunk_index),

                _ = delete_receiver.changed() => break Ok(self.chunk_index),
            }
        }
    }

    pub async fn download_chunk(&self) -> anyhow::Result<usize> {
        let bili_client = self.download_task.app.get_bili_client();
        let chunk_data = bili_client
            .get_media_chunk(&self.url, self.start, self.end)
            .await?;

        let len = chunk_data.len() as u64;
        self.download_task
            .app
            .get_download_manager()
            .byte_per_sec
            .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
        // 将下载的内容写入文件
        {
            let mut file = self.file.lock();
            file.seek(std::io::SeekFrom::Start(self.start))?;
            file.write_all(&chunk_data)?;
        }

        let chunk_download_interval_sec = self
            .download_task
            .app
            .get_config()
            .read()
            .chunk_download_interval_sec;
        sleep(Duration::from_secs(chunk_download_interval_sec)).await;

        Ok(self.chunk_index)
    }

    async fn acquire_chunk_permit<'a>(
        &'a self,
        permit: &mut Option<SemaphorePermit<'a>>,
    ) -> anyhow::Result<()> {
        *permit = match permit.take() {
            // 如果有permit，则直接用
            Some(permit) => Some(permit),
            // 如果没有permit，则获取permit
            None => Some(
                self.download_task
                    .app
                    .get_download_manager()
                    .inner()
                    .media_chunk_sem
                    .acquire()
                    .await?,
            ),
        };

        Ok(())
    }
}

#[derive(Clone)]
enum EpisodeInfo {
    Normal(NormalInfo),
    Bangumi(BangumiInfo, i64),
    Cheese(CheeseInfo, i64),
}

trait GetOrInitEpisodeInfo {
    async fn get_or_init<'a>(
        &'a mut self,
        bili_client: &BiliClient,
        aid: i64,
        ep_id: Option<i64>,
        episode_type: EpisodeType,
    ) -> anyhow::Result<&'a mut EpisodeInfo>;
}

impl GetOrInitEpisodeInfo for Option<EpisodeInfo> {
    async fn get_or_init<'a>(
        &'a mut self,
        bili_client: &BiliClient,
        aid: i64,
        ep_id: Option<i64>,
        episode_type: EpisodeType,
    ) -> anyhow::Result<&'a mut EpisodeInfo> {
        if let Some(info) = self {
            return Ok(info);
        }

        let new_info = match episode_type {
            EpisodeType::Normal => {
                let info = bili_client
                    .get_normal_info(GetNormalInfoParams::Aid(aid))
                    .await
                    .context("获取普通视频信息失败")?;
                EpisodeInfo::Normal(info)
            }
            EpisodeType::Bangumi => {
                let ep_id = ep_id.context("ep_id为None")?;
                let info = bili_client
                    .get_bangumi_info(GetBangumiInfoParams::EpId(ep_id))
                    .await
                    .context("获取番剧信息失败")?;
                EpisodeInfo::Bangumi(info, ep_id)
            }
            EpisodeType::Cheese => {
                let ep_id = ep_id.context("ep_id为None")?;
                let info = bili_client
                    .get_cheese_info(GetCheeseInfoParams::EpId(ep_id))
                    .await
                    .context("获取课程信息失败")?;
                EpisodeInfo::Cheese(info, ep_id)
            }
        };

        Ok(self.insert(new_info))
    }
}
