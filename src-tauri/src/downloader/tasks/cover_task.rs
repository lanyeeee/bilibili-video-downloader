use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    downloader::{download_progress::DownloadProgress, download_task::DownloadTask},
    extensions::AppHandleExt,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CoverTask {
    pub selected: bool,
    pub url: String,
    pub completed: bool,
}

impl CoverTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let bili_client = download_task.app.get_bili_client();
        let (cover_data, ext) = bili_client
            .get_cover_data_and_ext(&progress.cover_task.url)
            .await
            .context("获取封面失败")?;

        let save_path = episode_dir.join(format!("{filename}.{ext}"));
        std::fs::write(&save_path, cover_data)
            .context(format!("保存封面到`{}`失败", save_path.display()))?;

        download_task.update_progress(|p| p.cover_task.completed = true);

        Ok(())
    }
}
