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
    events::DownloadEvent,
    extensions::{AnyhowErrorToStringChain, AppHandleExt},
    types::create_download_task_params::CreateDownloadTaskParams,
    utils::{self},
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
            if let Err(err) = progress.save(app) {
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

        if let Err(err) = updated_progress.save(&self.app) {
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
