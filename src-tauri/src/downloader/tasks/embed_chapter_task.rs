use std::sync::Arc;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    downloader::{download_progress::DownloadProgress, download_task::DownloadTask},
    extensions::GetOrInitPlayerInfo,
    types::player_info::PlayerInfo,
    utils,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EmbedChapterTask {
    pub selected: bool,
    pub completed: bool,
}

impl EmbedChapterTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            download_task.update_progress(|p| p.embed_chapter_task.completed = true);
            return Ok(());
        }

        let ffmpeg_program = utils::get_ffmpeg_program().context("获取FFmpeg程序路径失败")?;
        let output_path = episode_dir.join(format!("{filename}-embed.mp4"));

        let player_info = player_info
            .get_or_init(&download_task.app, progress)
            .await?;

        let metadata_content = player_info.generate_chapter_metadata();
        let metadata_path = episode_dir.join(format!("{filename}.FFMETA.ini"));

        std::fs::write(&metadata_path, metadata_content)
            .context(format!("保存章节元数据到`{}`失败", metadata_path.display()))?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let video_path_clone = video_path.clone();
        let metadata_path_clone = metadata_path.clone();
        let output_path_clone = output_path.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut command = std::process::Command::new(ffmpeg_program);

            command
                .arg("-i")
                .arg(video_path_clone)
                .arg("-i")
                .arg(metadata_path_clone)
                .arg("-map_metadata")
                .arg("1")
                .arg("-c")
                .arg("copy")
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
        std::fs::rename(&output_path, &video_path).context(format!(
            "将`{}`重命名为`{}`失败",
            output_path.display(),
            video_path.display()
        ))?;
        std::fs::remove_file(&metadata_path).context(format!(
            "删除章节元数据文件`{}`失败",
            metadata_path.display()
        ))?;

        download_task.update_progress(|p| p.embed_chapter_task.completed = true);

        Ok(())
    }
}
