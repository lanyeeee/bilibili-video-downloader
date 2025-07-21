use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Context;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::sync::Semaphore;

use crate::{
    events::DownloadEvent,
    extensions::{AnyhowErrorToStringChain, AppHandleExt},
    types::create_download_task_params::CreateDownloadTaskParams,
};

use super::{
    download_progress::DownloadProgress, download_task::DownloadTask,
    download_task_state::DownloadTaskState,
};

pub struct DownloadManager {
    pub app: AppHandle,
    pub task_sem: Arc<Semaphore>,
    pub media_chunk_sem: Arc<Semaphore>,
    pub byte_per_sec: Arc<AtomicU64>,
    pub download_tasks: RwLock<HashMap<String, Arc<DownloadTask>>>,
}

impl DownloadManager {
    pub fn new(app: AppHandle) -> Self {
        let (task_concurrency, chunk_concurrency) = {
            let config = app.get_config().inner().read();
            (config.task_concurrency, config.chunk_concurrency)
        };

        let manager = Self {
            app,
            task_sem: Arc::new(Semaphore::new(task_concurrency)),
            media_chunk_sem: Arc::new(Semaphore::new(chunk_concurrency)),
            byte_per_sec: Arc::new(AtomicU64::new(0)),
            download_tasks: RwLock::new(HashMap::new()),
        };

        tauri::async_runtime::spawn(Self::emit_download_speed_loop(
            manager.app.clone(),
            manager.byte_per_sec.clone(),
        ));

        manager
    }

    pub fn restore_download_tasks(&self) -> anyhow::Result<()> {
        let task_dir = self.get_task_dir()?;
        std::fs::create_dir_all(&task_dir)
            .context(format!("创建下载任务目录`{}`失败", task_dir.display()))?;

        let mut tasks = self.download_tasks.write();
        for entry in std::fs::read_dir(&task_dir)?.filter_map(Result::ok) {
            let path = entry.path();
            let extension = path.extension().and_then(|s| s.to_str());
            if extension != Some("json") {
                // 如果这个文件不是json则删除
                let _ = std::fs::remove_file(&path);
                continue;
            }

            let progress_json = std::fs::read_to_string(&path)?;

            let progress: DownloadProgress =
                if let Ok(progress) = serde_json::from_str(&progress_json) {
                    progress
                } else {
                    // 如果这个json解析失败则删除
                    let _ = std::fs::remove_file(&path);
                    continue;
                };

            let new_task = DownloadTask::from_progress(self.app.clone(), progress);
            let old_task = tasks.insert(new_task.task_id.clone(), new_task);
            if let Some(old_task) = old_task {
                // 如果同一个ID的下载任务已经存在，则取消旧的任务
                let _ = old_task.cancel_sender.send(());
            }
        }

        Ok(())
    }

    pub fn create_download_tasks(&self, params: &CreateDownloadTaskParams) {
        let new_tasks = DownloadTask::from_params(&self.app, params);
        let mut tasks = self.download_tasks.write();
        for new_task in new_tasks {
            tasks.insert(new_task.task_id.clone(), new_task);
        }
    }

    pub fn pause_download_tasks(&self, task_ids: &Vec<String>) {
        let tasks = self.download_tasks.read();
        for task_id in task_ids {
            let Some(task) = tasks.get(task_id) else {
                let err_title = "暂停下载任务失败";
                let err_msg = format!("未找到ID为`{task_id}`的下载任务");
                tracing::error!(err_title, message = err_msg);
                continue;
            };
            task.set_state(DownloadTaskState::Paused);
            tracing::debug!("已将ID为`{task_id}`的下载任务状态设置为`Paused`");
        }
    }

    pub fn resume_download_tasks(&self, task_ids: &Vec<String>) {
        let tasks = self.download_tasks.read();
        for task_id in task_ids {
            let Some(task) = tasks.get(task_id) else {
                let err_title = "继续下载任务失败";
                let err_msg = format!("未找到ID为`{task_id}`的下载任务");
                tracing::error!(err_title, message = err_msg);
                continue;
            };
            task.set_state(DownloadTaskState::Pending);
            tracing::debug!("已将ID为`{task_id}`的下载任务状态设置为`Pending`");
        }
    }

    pub fn delete_download_tasks(&self, task_ids: &Vec<String>) {
        let mut tasks = self.download_tasks.write();
        for task_id in task_ids {
            let Some(task) = tasks.remove(task_id) else {
                let err_title = "删除下载任务失败";
                let err_msg = format!("未找到ID为`{task_id}`的下载任务");
                tracing::error!(err_title, message = err_msg);
                continue;
            };

            if let Err(err) = self.delete_progress_file(task_id) {
                let err_title = "删除下载任务失败";
                let err_msg = format!("删除ID为`{task_id}`的下载任务文件失败: {err}");
                tracing::error!(err_title, message = err_msg);
                tasks.insert(task_id.clone(), task);
                continue;
            }

            if let Err(err) = task.delete_sender.send(()).map_err(anyhow::Error::from) {
                let err_title = "删除下载任务失败";
                let err = err.context(format!("通知ID为`{task_id}`的下载任务删除失败"));
                let string_chain = err.to_string_chain();
                tracing::error!(err_title, message = string_chain);
                tasks.insert(task_id.clone(), task);
                continue;
            }

            tracing::debug!("已通知ID为`{task_id}`的下载任务删除");
        }
    }

    pub fn restart_download_tasks(&self, task_ids: &Vec<String>) {
        let tasks = self.download_tasks.read();
        for task_id in task_ids {
            let Some(task) = tasks.get(task_id) else {
                let err_title = "重来下载任务失败";
                let err_msg = format!("未找到ID为`{task_id}`的下载任务");
                tracing::error!(err_title, message = err_msg);
                continue;
            };

            if let Err(err) = task.restart_sender.send(()).map_err(anyhow::Error::from) {
                let err_title = "重来下载任务失败";
                let err = err.context(format!("通知ID为`{task_id}`的下载任务重来失败"));
                let string_chain = err.to_string_chain();
                tracing::error!(err_title, message = string_chain);
                continue;
            }

            tracing::debug!("已通知ID为`{task_id}`的下载任务重来");
        }
    }

    async fn emit_download_speed_loop(app: AppHandle, byte_per_sec: Arc<AtomicU64>) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let byte_per_sec = byte_per_sec.swap(0, Ordering::Relaxed);
            #[allow(clippy::cast_precision_loss)]
            let mega_byte_per_sec = byte_per_sec as f64 / 1024.0 / 1024.0;
            let speed = format!("{mega_byte_per_sec:.2}MB/s");
            let _ = DownloadEvent::Speed { speed }.emit(&app);
        }
    }

    fn get_task_dir(&self) -> anyhow::Result<PathBuf> {
        let app_data_dir = self.app.path().app_data_dir()?;
        let task_dir = app_data_dir.join(".下载任务");
        Ok(task_dir)
    }

    fn delete_progress_file(&self, task_id: &str) -> anyhow::Result<()> {
        let task_dir = self.get_task_dir()?;
        let task_file = task_dir.join(format!("{task_id}.json"));
        if task_file.exists() {
            std::fs::remove_file(task_file)?;
        }
        Ok(())
    }
}
