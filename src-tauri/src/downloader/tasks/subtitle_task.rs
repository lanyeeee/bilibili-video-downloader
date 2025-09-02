use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    downloader::{download_progress::DownloadProgress, download_task::DownloadTask},
    extensions::{AppHandleExt, GetOrInitPlayerInfo},
    types::player_info::PlayerInfo,
    utils,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SubtitleTask {
    pub selected: bool,
    pub completed: bool,
}

impl SubtitleTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let player_info = player_info
            .get_or_init(&download_task.app, progress)
            .await?;

        let bili_client = download_task.app.get_bili_client();

        for subtitle_detail in &player_info.subtitle.subtitles {
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

        download_task.update_progress(|p| p.subtitle_task.completed = true);

        Ok(())
    }
}
