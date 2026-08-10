use std::{io, path::PathBuf, sync::Arc};

use eyre::{WrapErr, eyre};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tracing::instrument;

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
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct VideoProcessTask {
    pub merge_selected: bool,
    pub embed_chapter_selected: bool,
    pub embed_skip_selected: bool,
    pub completed: bool,
    pub skipped: bool,
}

impl VideoProcessTask {
    pub fn mark_uncompleted(&mut self) {
        self.completed = false;
        self.skipped = false;
    }

    pub fn is_completed(&self) -> bool {
        !self.merge_selected && !self.embed_chapter_selected && !self.embed_skip_selected
            || self.completed
    }

    #[instrument(level = "error", skip_all)]
    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> eyre::Result<()> {
        let embed_selected = self.embed_chapter_selected || self.embed_skip_selected;

        if self.merge_selected && embed_selected {
            self.merge_and_embed(download_task, progress, player_info)
                .await
                .wrap_err("自动合并+嵌入章节元数据失败")?;
        } else if self.merge_selected {
            self.merge(download_task, progress)
                .await
                .wrap_err("自动合并失败")?;
        } else if embed_selected {
            self.embed(download_task, progress, player_info)
                .await
                .wrap_err("嵌入章节元数据失败")?;
        }

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn merge_and_embed(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> eyre::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let ffmpeg_program = utils::get_ffmpeg_program().wrap_err("获取FFmpeg程序路径失败")?;

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
                .wrap_err("嵌入章节元数据失败")?;
            return Ok(());
        }

        let metadata_path = self
            .create_chapter_metadata(&download_task.app, progress, player_info)
            .await
            .wrap_err("创建章节元数据失败")?;

        let output_path = episode_dir.join(format!("{filename}-merged.mp4"));

        let mut command = std::process::Command::new(ffmpeg_program);

        command.arg("-i").arg(&video_path);
        command.arg("-i").arg(&audio_path);
        if let Some(metadata_path) = &metadata_path {
            command.arg("-i").arg(metadata_path);
            command.arg("-map_metadata").arg("2");
        }

        command.arg("-c").arg("copy");
        command.arg("-map").arg("0:v:0");
        command.arg("-map").arg("1:a:0");
        command.arg(&output_path).arg("-y");

        #[cfg(target_os = "windows")]
        {
            // 隐藏窗口
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }

        execute_ffmpeg(command).await.wrap_err("执行FFmpeg失败")?;

        std::fs::remove_file(&video_path)
            .wrap_err(format!("删除视频文件`{}`失败", video_path.display()))?;
        std::fs::remove_file(&audio_path)
            .wrap_err(format!("删除音频文件`{}`失败", audio_path.display()))?;
        std::fs::rename(&output_path, &video_path).wrap_err(format!(
            "将`{}`重命名为`{}`失败",
            output_path.display(),
            video_path.display()
        ))?;

        if let Some(metadata_path) = metadata_path {
            std::fs::remove_file(&metadata_path).wrap_err(format!(
                "删除章节元数据文件`{}`失败",
                metadata_path.display()
            ))?;
        }

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn merge(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> eyre::Result<()> {
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

        let ffmpeg_program = utils::get_ffmpeg_program().wrap_err("获取FFmpeg程序路径失败")?;

        let mut command = std::process::Command::new(ffmpeg_program);

        command.arg("-i").arg(&video_path);
        command.arg("-i").arg(&audio_path);
        command.arg("-c").arg("copy");
        command.arg("-map").arg("0:v:0");
        command.arg("-map").arg("1:a:0");
        command.arg(&output_path).arg("-y");

        #[cfg(target_os = "windows")]
        {
            // 隐藏窗口
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }

        execute_ffmpeg(command).await.wrap_err("执行FFmpeg失败")?;

        std::fs::remove_file(&video_path)
            .wrap_err(format!("删除视频文件`{}`失败", video_path.display()))?;
        std::fs::remove_file(&audio_path)
            .wrap_err(format!("删除音频文件`{}`失败", audio_path.display()))?;
        std::fs::rename(&output_path, &video_path).wrap_err(format!(
            "将`{}`重命名为`{}`失败",
            output_path.display(),
            video_path.display()
        ))?;

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn embed(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> eyre::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let ffmpeg_program = utils::get_ffmpeg_program().wrap_err("获取FFmpeg程序路径失败")?;

        let video_path = episode_dir.join(format!("{filename}.mp4"));
        if !video_path.exists() {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        }

        let output_path = episode_dir.join(format!("{filename}-embed.mp4"));

        let metadata_path = self
            .create_chapter_metadata(&download_task.app, progress, player_info)
            .await
            .wrap_err("创建章节元数据失败")?;

        let Some(metadata_path) = metadata_path else {
            download_task.update_progress(|p| p.video_process_task.completed = true);
            return Ok(());
        };

        let mut command = std::process::Command::new(ffmpeg_program);

        command.arg("-i").arg(&video_path);
        command.arg("-i").arg(&metadata_path);
        command.arg("-map_metadata").arg("1");
        command.arg("-c").arg("copy");
        command.arg(&output_path).arg("-y");

        #[cfg(target_os = "windows")]
        {
            // 隐藏窗口
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }

        execute_ffmpeg(command).await.wrap_err("执行FFmpeg失败")?;

        std::fs::remove_file(&video_path)
            .wrap_err(format!("删除视频文件`{}`失败", video_path.display()))?;
        std::fs::rename(&output_path, &video_path).wrap_err(format!(
            "将`{}`重命名为`{}`失败",
            output_path.display(),
            video_path.display()
        ))?;
        std::fs::remove_file(&metadata_path).wrap_err(format!(
            "删除章节元数据文件`{}`失败",
            metadata_path.display()
        ))?;

        download_task.update_progress(|p| p.video_process_task.completed = true);

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn create_chapter_metadata(
        &self,
        app: &AppHandle,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> eyre::Result<Option<PathBuf>> {
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
            .wrap_err(format!("保存章节元数据到`{}`失败", metadata_path.display()))?;

        Ok(Some(metadata_path))
    }
}

#[instrument(level = "error", skip_all)]
async fn execute_ffmpeg(mut command: std::process::Command) -> eyre::Result<()> {
    let span = tracing::Span::current();

    tauri::async_runtime::spawn_blocking(move || {
        let _enter = span.enter();

        let program = command.get_program().to_string_lossy().into_owned();
        let output = command.output().map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                eyre!(err).wrap_err(format!("找不到FFmpeg `{program}`"))
            } else {
                eyre!(err)
            }
        })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(eyre!(format!("STDOUT: {stdout}")))
            .wrap_err(format!("STDERR: {stderr}"))
            .wrap_err("原因可能是视频或音频文件损坏，建议[重来]试试")
    })
    .await?
}
