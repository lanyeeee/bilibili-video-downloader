use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::downloader::{
    download_progress::DownloadProgress,
    download_task::DownloadTask,
    episode_info::{EpisodeInfo, GetOrInitEpisodeInfo},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct JsonTask {
    pub selected: bool,
    pub completed: bool,
}

impl JsonTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        episode_info: &mut Option<EpisodeInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let episode_info = episode_info
            .get_or_init(&download_task.app, progress)
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

        download_task.update_progress(|p| p.json_task.completed = true);

        Ok(())
    }
}
