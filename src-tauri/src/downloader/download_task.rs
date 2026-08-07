use std::{sync::Arc, time::Duration};

use eyre::WrapErr;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::{
    sync::{SemaphorePermit, watch},
    time::sleep,
};
use tracing::instrument;

use crate::{
    downloader::episode_type::EpisodeType,
    events::DownloadEvent,
    extensions::{AppHandleExt, EyreReportToMessage},
    types::{
        create_download_task_params::CreateDownloadTaskParams,
        restart_download_task_params::RestartDownloadTaskParams,
    },
};

use super::{download_progress::DownloadProgress, download_task_state::DownloadTaskState};

pub struct DownloadTask {
    pub app: AppHandle,
    pub state_sender: watch::Sender<DownloadTaskState>,
    pub restart_sender: watch::Sender<()>,
    pub cancel_sender: watch::Sender<()>,
    pub delete_sender: watch::Sender<()>,
    pub task_id: String,
    pub trace_fields: DownloadTaskTraceFields,
    pub progress: RwLock<DownloadProgress>,
}

impl DownloadTask {
    #[allow(clippy::too_many_lines)]
    #[instrument(level = "error", skip_all)]
    pub fn from_params(app: &AppHandle, params: &CreateDownloadTaskParams) -> Vec<Arc<Self>> {
        use CreateDownloadTaskParams::{Bangumi, Cheese, Normal};

        let mut progresses = Vec::new();
        match params {
            Normal(params) => {
                for &(aid, cid) in &params.aid_cid_pairs {
                    let span = tracing::error_span!(
                        "from_params_normal",
                        aid = aid,
                        bvid = params.info.bvid,
                        cid = cid,
                        collection_title = params.info.title,
                        up_name = params.info.owner.name,
                        up_uid = params.info.owner.mid,
                    );
                    let _enter = span.enter();

                    let progress = match DownloadProgress::from_normal(app, &params.info, aid, cid)
                    {
                        Ok(progress) => progress,
                        Err(err) => {
                            let err_title = "创建普通视频的下载进度失败";
                            let message = err.to_message();
                            tracing::error!(err_title, message);
                            continue;
                        }
                    };

                    progresses.extend(progress);
                }
            }
            Bangumi(params) => {
                for ep_id in &params.ep_ids {
                    let span = tracing::error_span!(
                        "from_params_bangumi",
                        ep_id = ep_id,
                        collection_title = params.info.title,
                        up_name = params.info.up_info.as_ref().map(|up_info| &up_info.uname),
                        up_uid = params.info.up_info.as_ref().map(|up_info| up_info.mid),
                    );
                    let _enter = span.enter();

                    let progress = match DownloadProgress::from_bangumi(app, &params.info, *ep_id) {
                        Ok(progress) => progress,
                        Err(err) => {
                            let err_title = "创建番剧的下载进度失败";
                            let message = err.to_message();
                            tracing::error!(err_title, message);
                            continue;
                        }
                    };

                    progresses.push(progress);
                }
            }
            Cheese(params) => {
                for ep_id in &params.ep_ids {
                    let span = tracing::error_span!(
                        "from_params_cheese",
                        ep_id = ep_id,
                        collection_title = params.info.title,
                        up_name = params.info.up_info.uname,
                        up_uid = params.info.up_info.mid,
                    );
                    let _enter = span.enter();

                    let progress = match DownloadProgress::from_cheese(app, &params.info, *ep_id) {
                        Ok(progress) => progress,
                        Err(err) => {
                            let err_title = "创建课程的下载进度失败";
                            let message = err.to_message();
                            tracing::error!(err_title, message);
                            continue;
                        }
                    };

                    progresses.push(progress);
                }
            }
        }

        let mut tasks = Vec::new();
        for progress in progresses {
            let span = tracing::error_span!(
                "create_tasks",
                task_id = progress.task_id,
                episode_type = ?progress.episode_type,
                aid = progress.aid,
                bvid = progress.bvid,
                cid = progress.cid,
                ep_id = progress.ep_id,
                collection_title = progress.collection_title,
                episode_title = progress.episode_title,
                episode_title_in_collection = progress.episode_title_in_collection,
                episode_order = progress.episode_order,
                part_title = progress.part_title,
                part_order = progress.part_order,
                up_name = progress.up_name,
                up_uid = progress.up_uid,
            );
            let _enter = span.enter();

            if let Err(err) = progress.save(app, true) {
                let err_title = "保存下载进度到文件失败";
                let message = err.to_message();
                tracing::error!(err_title, message);
            }

            let auto_start = app.get_config().read().auto_start_download_task;
            let init_state = if auto_start {
                DownloadTaskState::Pending
            } else {
                DownloadTaskState::Paused
            };

            let (state_sender, _) = watch::channel(init_state);
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
                trace_fields: DownloadTaskTraceFields::from(&progress),
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
            trace_fields: DownloadTaskTraceFields::from(&progress),
            progress: RwLock::new(progress),
        });

        tauri::async_runtime::spawn(task.clone().process());

        task
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn pause(&self) {
        self.set_state(DownloadTaskState::Paused);
        tracing::debug!("已将下载任务状态设置为`Paused`");
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn resume(&self) {
        self.set_state(DownloadTaskState::Pending);
        tracing::debug!("已将下载任务状态设置为`Pending`");
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn cancel(&self) {
        let _ = self.cancel_sender.send(());
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn delete(&self) -> eyre::Result<()> {
        self.delete_progress_file()?;
        self.delete_sender
            .send(())
            .map_err(eyre::Report::from)
            .wrap_err("通知ID对应的下载任务删除失败")?;
        tracing::debug!("已通知ID对应的下载任务删除");
        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn restart(&self) -> eyre::Result<()> {
        self.restart_sender
            .send(())
            .map_err(eyre::Report::from)
            .wrap_err("通知ID对应的下载任务重来失败")?;
        tracing::debug!("已通知ID对应的下载任务重来");
        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    pub fn restart_with_params(&self, params: &RestartDownloadTaskParams) -> eyre::Result<()> {
        {
            let mut progress = self.progress.write();

            progress.video_task.selected = params.video_task_selected;
            progress.audio_task.selected = params.audio_task_selected;
            progress.video_process_task.merge_selected = params.merge_selected;
            progress.video_process_task.embed_chapter_selected = params.embed_chapter_selected;
            progress.video_process_task.embed_skip_selected = params.embed_skip_selected;
            progress.subtitle_task.selected = params.subtitle_task_selected;
            progress.danmaku_task.xml_selected = params.xml_danmaku_selected;
            progress.danmaku_task.ass_selected = params.ass_danmaku_selected;
            progress.danmaku_task.json_selected = params.json_danmaku_selected;
            progress.cover_task.selected = params.cover_task_selected;
            progress.nfo_task.selected = params.nfo_task_selected;
            progress.json_task.selected = params.json_task_selected;

            progress.video_task.video_quality = params.video_quality;
            progress.video_task.codec_type = params.codec_type;
            progress.audio_task.audio_quality = params.audio_quality;
        }

        self.restart()
    }

    #[instrument(
        level = "error",
        skip_all,
        fields(
            task_id = self.trace_fields.task_id,
            episode_type = ?self.trace_fields.episode_type,
            aid = self.trace_fields.aid,
            bvid = self.trace_fields.bvid,
            cid = self.trace_fields.cid,
            ep_id = self.trace_fields.ep_id,
            collection_title = self.trace_fields.collection_title,
            episode_title = self.trace_fields.episode_title,
            episode_title_in_collection = self.trace_fields.episode_title_in_collection,
            episode_order = self.trace_fields.episode_order,
            part_title = self.trace_fields.part_title,
            part_order = self.trace_fields.part_order,
            up_name = self.trace_fields.up_name,
            up_uid = self.trace_fields.up_uid,
        )
    )]
    async fn process(self: Arc<Self>) {
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
                    .get_or_insert_with(|| Box::pin(self.download()))
                    .await;
            };

            tokio::select! {
                () = download_task, if state_is_downloading && permit.is_some() => {
                    download_task_option = None;
                    if let Some(permit) = permit.take() {
                        drop(permit);
                    }
                }

                () = self.acquire_task_permit(&mut permit), if state_is_pending => {},

                _ = state_receiver.changed() => {
                    self.handle_state_change(&mut permit, &mut state_receiver).await;
                }

                _ = restart_receiver.changed() => {
                    self.handle_restart_notify();
                    tracing::debug!("下载任务已重来");
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

                    tracing::debug!("下载任务已删除");
                    return;
                }
            }
        }
    }

    #[instrument(level = "error", skip_all)]
    async fn download(self: &Arc<Self>) {
        let mut progress = self.progress.read().clone();

        if progress.is_completed() {
            tracing::info!("跳过下载，因为下载任务已完成");
            self.set_state(DownloadTaskState::Completed);
            return;
        }

        tracing::debug!("开始下载");
        if let Err(err) = progress
            .process(self)
            .await
            .wrap_err("[继续]失败的任务可以断点续传")
        {
            let err_title = "下载失败";
            let message = err.to_message();
            tracing::error!(err_title, message);

            self.set_state(DownloadTaskState::Failed);

            return;
        }

        self.sleep_between_task().await;

        self.set_state(DownloadTaskState::Completed);
        tracing::info!("下载成功");
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

    #[instrument(level = "error", skip_all)]
    async fn acquire_task_permit<'a>(&'a self, permit: &mut Option<SemaphorePermit<'a>>) {
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
                .map_err(eyre::Report::from)
            {
                Ok(permit) => Some(permit),
                Err(err) => {
                    let err_title = "获取下载任务的permit失败";
                    let message = err.to_message();
                    tracing::error!(err_title, message);

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
            .map_err(eyre::Report::from)
        {
            let err_title = "发送状态`Downloading`失败";
            let message = err.to_message();
            tracing::error!(err_title, message);

            self.set_state(DownloadTaskState::Failed);
        }
    }

    #[instrument(level = "error", skip_all)]
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
            tracing::debug!("下载任务已暂停");
            if let Some(permit) = permit.take() {
                drop(permit);
            }
        }
    }

    #[instrument(level = "error", skip_all)]
    fn handle_restart_notify(&self) {
        self.update_progress(|p| {
            p.mark_uncompleted();
        });
        self.set_state(DownloadTaskState::Pending);
    }

    #[instrument(
        level = "error",
        skip_all,
        fields(
            task_id = self.trace_fields.task_id,
            episode_type = ?self.trace_fields.episode_type,
            aid = self.trace_fields.aid,
            bvid = self.trace_fields.bvid,
            cid = self.trace_fields.cid,
            ep_id = self.trace_fields.ep_id,
            collection_title = self.trace_fields.collection_title,
            episode_title = self.trace_fields.episode_title,
            episode_title_in_collection = self.trace_fields.episode_title_in_collection,
            episode_order = self.trace_fields.episode_order,
            part_title = self.trace_fields.part_title,
            part_order = self.trace_fields.part_order,
            up_name = self.trace_fields.up_name,
            up_uid = self.trace_fields.up_uid,
        )
    )]
    fn set_state(&self, state: DownloadTaskState) {
        if let Err(err) = self.state_sender.send(state).map_err(eyre::Report::from) {
            let err_title = format!("发送状态`{state:?}`失败");
            let message = err.to_message();
            tracing::error!(err_title, message);
        }
    }

    #[instrument(level = "error", skip_all)]
    pub fn update_progress(&self, update_fn: impl FnOnce(&mut DownloadProgress)) {
        // 修改数据
        let updated_progress = {
            let mut progress = self.progress.write();
            update_fn(&mut progress);
            // TODO: 这里应该返回 progress.clone()
            // 专门用一个 {} 框出来就是为了避免在emit和save期间仍持有写锁
            // 然而这里弄错了progress的类型
            // 错把progress当成了DownloadProgress，实则类型为RwLockWriteGuard
            progress
        };
        // 发送更新事件并保存到文件
        let _ = DownloadEvent::ProgressUpdate {
            progress: updated_progress.clone(),
        }
        .emit(&self.app);

        if let Err(err) = updated_progress.save(&self.app, false) {
            let err_title = "保存下载进度到文件失败";
            let message = err.to_message();
            tracing::error!(err_title, message);
        }
    }

    #[instrument(level = "error", skip_all, fields(task_id = self.task_id))]
    fn delete_progress_file(&self) -> eyre::Result<()> {
        let app_data_dir = self.app.path().app_data_dir()?;
        let task_file = app_data_dir
            .join(".下载任务")
            .join(format!("{}.json", self.task_id));
        if task_file.exists() {
            std::fs::remove_file(task_file)?;
        }
        Ok(())
    }
}

pub struct DownloadTaskTraceFields {
    pub task_id: String,
    pub episode_type: EpisodeType,
    pub aid: i64,
    pub bvid: Option<String>,
    pub cid: i64,
    pub ep_id: Option<i64>,
    pub collection_title: String,
    pub episode_title: String,
    pub episode_title_in_collection: Option<String>,
    pub episode_order: i64,
    pub part_title: Option<String>,
    pub part_order: Option<i64>,
    pub up_name: Option<String>,
    pub up_uid: Option<i64>,
}

impl From<&DownloadProgress> for DownloadTaskTraceFields {
    fn from(progress: &DownloadProgress) -> Self {
        Self {
            task_id: progress.task_id.clone(),
            episode_type: progress.episode_type,
            aid: progress.aid,
            bvid: progress.bvid.clone(),
            cid: progress.cid,
            ep_id: progress.ep_id,
            collection_title: progress.collection_title.clone(),
            episode_title: progress.episode_title.clone(),
            episode_title_in_collection: progress.episode_title_in_collection.clone(),
            episode_order: progress.episode_order,
            part_title: progress.part_title.clone(),
            part_order: progress.part_order,
            up_name: progress.up_name.clone(),
            up_uid: progress.up_uid,
        }
    }
}
