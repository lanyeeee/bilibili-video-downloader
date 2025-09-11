use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::{
    downloader::{
        chapter_segments::{ChapterSegment, ChapterSegments},
        download_progress::DownloadProgress,
        download_task::DownloadTask,
    },
    extensions::{AppHandleExt, GetOrInitPlayerInfo},
    types::player_info::PlayerInfo,
    utils,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
pub struct VideoProcessTask {
    pub merge_selected: bool,
    pub embed_chapter_selected: bool,
    pub embed_skip_selected: bool,
    pub completed: bool,
}

impl VideoProcessTask {
    pub fn is_completed(&self) -> bool {
        !self.merge_selected && !self.embed_chapter_selected && !self.embed_skip_selected
            || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<()> {
        let embed_selected = self.embed_chapter_selected || self.embed_skip_selected;

        if self.merge_selected && embed_selected {
            self.merge_and_embed(download_task, progress, player_info)
                .await
                .context("自动合并+嵌入章节元数据失败")?;
        } else if self.merge_selected {
            println!("merge1");
            self.merge(download_task, progress)
                .await
                .context("自动合并失败")?;
        } else if embed_selected {
            self.embed(download_task, progress, player_info)
                .await
                .context("嵌入章节元数据失败")?;
        }

        Ok(())
    }

    async fn merge_and_embed(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let ffmpeg_program = utils::get_ffmpeg_program().context("获取FFmpeg程序路径失败")?;

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        }

        let audio_path = episode_dir.join(format!("{filename}.m4a"));
        if !audio_path.exists() {
            // 如果音频文件不存在，则只嵌入章节元数据
            self.embed(download_task, progress, player_info)
                .await
                .context("嵌入章节元数据失败")?;
            return Ok(());
        }

        let metadata_path = self
            .create_chapter_metadata(&download_task.app, progress, player_info)
            .await
            .context("创建章节元数据失败")?;

        let output_path = episode_dir.join(format!("{filename}-merged.mp4"));

        let (tx, rx) = tokio::sync::oneshot::channel();
        let video_path_clone = video_path.clone();
        let audio_path_clone = audio_path.clone();
        let metadata_path_clone = metadata_path.clone();
        let output_path_clone = output_path.clone();

        tokio::spawn(async move {
            let mut command = std::process::Command::new(ffmpeg_program);

            command.arg("-i").arg(video_path_clone);
            command.arg("-i").arg(audio_path_clone);
            if let Some(metadata_path) = metadata_path_clone {
                command.arg("-i").arg(metadata_path);
                command.arg("-map_metadata").arg("2");
            }

            command.arg("-c").arg("copy");
            command.arg("-map").arg("0:v:0");
            command.arg("-map").arg("1:a:0");

            command.arg(output_path_clone).arg("-y");

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

        if let Some(metadata_path) = metadata_path {
            std::fs::remove_file(&metadata_path).context(format!(
                "删除章节元数据文件`{}`失败",
                metadata_path.display()
            ))?;
        }

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    async fn merge(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        }

        let audio_path = episode_dir.join(format!("{filename}.m4a"));
        if !audio_path.exists() {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        }

        let output_path = episode_dir.join(format!("{filename}-merged.mp4"));

        let ffmpeg_program = utils::get_ffmpeg_program().context("获取FFmpeg程序路径失败")?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let video_path_clone = video_path.clone();
        let audio_path_clone = audio_path.clone();
        let output_path_clone = output_path.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut command = std::process::Command::new(ffmpeg_program);

            command.arg("-i").arg(video_path_clone);
            command.arg("-i").arg(audio_path_clone);

            command.arg("-c").arg("copy");
            command.arg("-map").arg("0:v:0");
            command.arg("-map").arg("1:a:0");

            command.arg(output_path_clone).arg("-y");

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

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    async fn embed(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let ffmpeg_program = utils::get_ffmpeg_program().context("获取FFmpeg程序路径失败")?;

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        }

        let output_path = episode_dir.join(format!("{filename}-embed.mp4"));

        let metadata_path = self
            .create_chapter_metadata(&download_task.app, progress, player_info)
            .await
            .context("创建章节元数据失败")?;

        let Some(metadata_path) = metadata_path else {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        };

        let (tx, rx) = tokio::sync::oneshot::channel();
        let video_path_clone = video_path.clone();
        let metadata_path_clone = metadata_path.clone();
        let output_path_clone = output_path.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut command = std::process::Command::new(ffmpeg_program);

            command.arg("-i").arg(video_path_clone);
            command.arg("-i").arg(metadata_path_clone);

            command.arg("-map_metadata").arg("1");
            command.arg("-c").arg("copy");

            command.arg(output_path_clone).arg("-y");

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

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    async fn create_chapter_metadata(
        &self,
        app: &AppHandle,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> anyhow::Result<Option<PathBuf>> {
        let mut chapter_segments = ChapterSegments {
            segments: Vec::new(),
        };

        if self.embed_chapter_selected {
            let player_info = player_info.get_or_init(app, progress).await?;
            let segments = player_info
                .view_points
                .iter()
                .map(|vp| ChapterSegment {
                    title: vp.content.clone(),
                    start: vp.from,
                    end: vp.to,
                })
                .collect();
            chapter_segments = ChapterSegments { segments };
        }

        if let (true, Some(bvid)) = (self.embed_skip_selected, &progress.bvid) {
            let bili_client = app.get_bili_client();
            let cid = Some(progress.cid);

            let skip_segments = bili_client.get_skip_segments(bvid, cid).await?;
            for segment in skip_segments.0 {
                if let Some(chapter_segment) = segment.into_chapter_segment() {
                    chapter_segments.insert(chapter_segment);
                }
            }
        }

        if chapter_segments.segments.is_empty() {
            return Ok(None);
        }

        let metadata_content = chapter_segments.generate_chapter_metadata(progress.duration);
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);
        let metadata_path = episode_dir.join(format!("{filename}.FFMETA.ini"));
        std::fs::write(&metadata_path, metadata_content)
            .context(format!("保存章节元数据到`{}`失败", metadata_path.display()))?;

        Ok(Some(metadata_path))
    }
}
