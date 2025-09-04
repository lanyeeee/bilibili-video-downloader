use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    sync::Arc,
};

use anyhow::{anyhow, Context};
use fs4::fs_std::FileExt;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tokio::task::JoinSet;

use crate::{
    downloader::{
        download_chunk_task::DownloadChunkTask, download_progress::DownloadProgress,
        download_task::DownloadTask, media_chunk::MediaChunk,
    },
    extensions::{AnyhowErrorToStringChain, AppHandleExt},
    types::{
        bangumi_media_url::BangumiMediaUrl, cheese_media_url::CheeseMediaUrl,
        codec_type::CodecType, normal_media_url::NormalMediaUrl, video_quality::VideoQuality,
    },
    utils,
};

const CHUNK_SIZE: u64 = 2 * 1024 * 1024; // 2MB

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct VideoTask {
    pub selected: bool,
    pub url: String,
    pub video_quality: VideoQuality,
    pub codec_type: CodecType,
    pub content_length: u64,
    pub chunks: Vec<MediaChunk>,
    pub completed: bool,
}

impl VideoTask {
    pub async fn prepare_normal(
        &mut self,
        app: &AppHandle,
        media_url: &NormalMediaUrl,
    ) -> anyhow::Result<()> {
        let mut join_set = JoinSet::new();

        for media in &media_url.dash.video {
            let app = app.clone();
            let id = media.id;
            let codecid = media.codecid;

            let mut urls = Vec::new();
            urls.extend_from_slice(&media.backup_url);
            urls.push(media.base_url.clone());

            join_set.spawn(async move {
                let bili_client = app.get_bili_client();
                let url_with_content_length = bili_client.get_url_with_content_length(urls).await;
                MediaForPrepare {
                    id,
                    url_with_content_length,
                    codecid,
                }
            });
        }

        let mut medias: Vec<MediaForPrepare> = Vec::new();

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        self.prepare(app, medias)?;

        Ok(())
    }

    pub async fn prepare_bangumi(
        &mut self,
        app: &AppHandle,
        media_url: &BangumiMediaUrl,
    ) -> anyhow::Result<()> {
        let mut medias: Vec<MediaForPrepare> = Vec::new();

        let mut join_set = JoinSet::new();

        if let Some(dash) = &media_url.dash {
            for media in &dash.video {
                let app = app.clone();
                let id = media.id;
                let codecid = media.codecid;

                let mut urls = Vec::new();
                urls.extend_from_slice(&media.backup_url);
                urls.push(media.base_url.clone());

                join_set.spawn(async move {
                    let bili_client = app.get_bili_client();
                    let url_with_content_length =
                        bili_client.get_url_with_content_length(urls).await;
                    MediaForPrepare {
                        id,
                        url_with_content_length,
                        codecid,
                    }
                });
            }
        }

        for durl in &media_url.durls {
            for media in &durl.durl {
                let app = app.clone();
                let id = durl.quality;
                let codecid = media_url.video_codecid;

                let mut urls = Vec::new();
                urls.extend_from_slice(&media.backup_url);
                urls.push(media.url.clone());

                join_set.spawn(async move {
                    let bili_client = app.get_bili_client();
                    let url_with_content_length =
                        bili_client.get_url_with_content_length(urls).await;
                    MediaForPrepare {
                        id,
                        url_with_content_length,
                        codecid,
                    }
                });
            }
        }

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        self.prepare(app, medias)?;

        Ok(())
    }

    pub async fn prepare_cheese(
        &mut self,
        app: &AppHandle,
        media_url: &CheeseMediaUrl,
    ) -> anyhow::Result<()> {
        let mut medias: Vec<MediaForPrepare> = Vec::new();

        let mut join_set = JoinSet::new();

        if let Some(dash) = &media_url.dash {
            for media in &dash.video {
                let app = app.clone();
                let id = media.id;
                let codecid = media.codecid;

                let mut urls = Vec::new();
                urls.extend_from_slice(&media.backup_url);
                urls.push(media.base_url.clone());

                join_set.spawn(async move {
                    let bili_client = app.get_bili_client();
                    let url_with_content_length =
                        bili_client.get_url_with_content_length(urls).await;
                    MediaForPrepare {
                        id,
                        url_with_content_length,
                        codecid,
                    }
                });
            }
        }

        for durl in &media_url.durls {
            for media in &durl.durl {
                let app = app.clone();
                let id = durl.quality;
                let codecid = media_url.video_codecid;

                let mut urls = Vec::new();
                urls.extend_from_slice(&media.backup_url);
                urls.push(media.url.clone());

                join_set.spawn(async move {
                    let bili_client = app.get_bili_client();
                    let url_with_content_length =
                        bili_client.get_url_with_content_length(urls).await;
                    MediaForPrepare {
                        id,
                        url_with_content_length,
                        codecid,
                    }
                });
            }
        }

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        self.prepare(app, medias)?;

        Ok(())
    }

    fn prepare(&mut self, app: &AppHandle, mut medias: Vec<MediaForPrepare>) -> anyhow::Result<()> {
        if medias.is_empty() {
            return Err(anyhow!("获取音频地址失败"));
        }

        let (video_quality_priority, codec_type_priority) = {
            let config = app.get_config().inner().read();
            (
                config.video_quality_priority.clone(),
                config.codec_type_priority.clone(),
            )
        };

        let video_priority_map: HashMap<&VideoQuality, usize> = video_quality_priority
            .iter()
            .enumerate()
            .map(|(index, quality)| (quality, index))
            .collect();
        medias.sort_by_key(|media| {
            let quality: VideoQuality = media.id.into();
            video_priority_map.get(&quality).unwrap_or(&usize::MAX)
        });

        let retain_id = medias[0].id;
        medias.retain(|m| m.id == retain_id);

        let codec_priority_map: HashMap<&CodecType, usize> = codec_type_priority
            .iter()
            .enumerate()
            .map(|(index, codec_type)| (codec_type, index))
            .collect();

        medias.sort_by_key(|m| {
            let codec_type: CodecType = m.codecid.into();
            codec_priority_map.get(&codec_type).unwrap_or(&usize::MAX)
        });

        let media = &medias[0];

        self.video_quality = media.id.into();
        self.codec_type = media.codecid.into();

        let (url, content_length) = media
            .url_with_content_length
            .iter()
            .find(|(url, _)| url.starts_with("https://upos-"))
            .unwrap_or(&media.url_with_content_length[0])
            .clone();

        self.url = url;

        if self.content_length != content_length {
            let chunk_count = content_length.div_ceil(CHUNK_SIZE);

            #[allow(clippy::cast_possible_truncation)]
            let mut chunks = Vec::with_capacity(chunk_count as usize);
            for i in 0..chunk_count {
                let start = i * CHUNK_SIZE;
                let end = std::cmp::min(start + CHUNK_SIZE, content_length) - 1;
                chunks.push(MediaChunk {
                    start,
                    end,
                    completed: false,
                });
            }

            self.content_length = content_length;
            self.chunks = chunks;
        }

        Ok(())
    }

    pub fn mark_uncompleted(&mut self) {
        self.completed = false;
        self.chunks.iter_mut().for_each(|chunk| {
            chunk.completed = false;
        });
    }

    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> anyhow::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let temp_file_path = episode_dir.join(format!(
            "{filename}.mp4.com.lanyeeee.bilibili-video-downloader"
        ));

        let (video_task, episode_title, ids_string) = {
            let progress = download_task.progress.read();
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
                download_task: download_task.clone(),
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
                Ok(i) => download_task.update_progress(|p| p.video_task.chunks[i].completed = true),
                Err(err) => {
                    let err_title = format!("{ids_string} `{episode_title}`视频的一个分片下载失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                }
            }
        }
        // 检查视频是否已下载完成
        let download_completed = download_task
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
            download_task.update_progress(|p| p.video_task.mark_uncompleted());
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

        download_task.update_progress(|p| p.video_task.completed = true);

        Ok(())
    }
}

struct MediaForPrepare {
    pub id: i64,
    pub url_with_content_length: Vec<(String, u64)>,
    pub codecid: i64,
}
