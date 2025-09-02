use std::{
    cmp::Reverse,
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
        audio_quality::AudioQuality, bangumi_media_url::BangumiMediaUrl,
        cheese_media_url::CheeseMediaUrl, normal_media_url::NormalMediaUrl,
    },
    utils,
};

const CHUNK_SIZE: u64 = 2 * 1024 * 1024; // 2MB

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AudioTask {
    pub selected: bool,
    pub url: String,
    pub audio_quality: AudioQuality,
    pub content_length: u64,
    pub chunks: Vec<MediaChunk>,
    pub completed: bool,
}

impl AudioTask {
    pub async fn prepare_normal(
        &mut self,
        app: &AppHandle,
        media_url: &NormalMediaUrl,
    ) -> anyhow::Result<()> {
        let mut join_set = JoinSet::new();

        if let Some(medias) = &media_url.dash.audio {
            for media in medias {
                let app = app.clone();
                let id = media.id;

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
                    }
                });
            }
        }

        if let Some(medias) = &media_url.dash.dolby.audio {
            for media in medias {
                let app = app.clone();
                let id = media.id;

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
                    }
                });
            }
        }

        let flac = media_url.dash.flac.as_ref();
        if let Some(media) = flac.and_then(|flac| flac.audio.as_ref()) {
            let app = app.clone();
            let id = media.id;

            let mut urls = Vec::new();
            urls.extend_from_slice(&media.backup_url);
            urls.push(media.base_url.clone());

            join_set.spawn(async move {
                let bili_client = app.get_bili_client();
                let url_with_content_length = bili_client.get_url_with_content_length(urls).await;
                MediaForPrepare {
                    id,
                    url_with_content_length,
                }
            });
        }

        let mut medias: Vec<MediaForPrepare> = Vec::new();

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        if medias.is_empty() {
            return Err(anyhow!("获取音频地址失败"));
        }

        self.prepare(app, medias);

        Ok(())
    }

    pub async fn prepare_bangumi(
        &mut self,
        app: &AppHandle,
        media_url: &BangumiMediaUrl,
    ) -> anyhow::Result<()> {
        let Some(dash) = &media_url.dash else {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        };

        let Some(medias) = &dash.audio else {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        };

        if medias.is_empty() {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        }

        let mut join_set = JoinSet::new();

        for media in medias {
            let app = app.clone();
            let id = media.id;

            let mut urls = Vec::new();
            urls.extend_from_slice(&media.backup_url);
            urls.push(media.base_url.clone());

            join_set.spawn(async move {
                let bili_client = app.get_bili_client();
                let url_with_content_length = bili_client.get_url_with_content_length(urls).await;
                MediaForPrepare {
                    id,
                    url_with_content_length,
                }
            });
        }

        let mut medias: Vec<MediaForPrepare> = Vec::new();

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        if medias.is_empty() {
            return Err(anyhow!("获取音频地址失败"));
        }

        self.prepare(app, medias);

        Ok(())
    }

    pub async fn prepare_cheese(
        &mut self,
        app: &AppHandle,
        media_url: &CheeseMediaUrl,
    ) -> anyhow::Result<()> {
        let Some(dash) = &media_url.dash else {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        };

        let Some(medias) = &dash.audio else {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        };

        if medias.is_empty() {
            // 如果没有音频，则直接返回
            self.completed = true;
            return Ok(());
        }

        let mut join_set = JoinSet::new();

        for media in medias {
            let app = app.clone();
            let id = media.id;

            let mut urls = Vec::new();
            urls.extend_from_slice(&media.backup_url);
            urls.push(media.base_url.clone());

            join_set.spawn(async move {
                let bili_client = app.get_bili_client();
                let url_with_content_length = bili_client.get_url_with_content_length(urls).await;
                MediaForPrepare {
                    id,
                    url_with_content_length,
                }
            });
        }

        let mut medias: Vec<MediaForPrepare> = Vec::new();

        while let Some(Ok(media)) = join_set.join_next().await {
            if !media.url_with_content_length.is_empty() {
                medias.push(media);
            }
        }

        if medias.is_empty() {
            return Err(anyhow!("获取音频地址失败"));
        }

        self.prepare(app, medias);

        Ok(())
    }

    fn prepare(&mut self, app: &AppHandle, mut medias: Vec<MediaForPrepare>) {
        medias.sort_by_key(|m| Reverse(m.id.to_audio_quality_for_prepare()));
        let best_quality_id = medias[0].id;

        let prefer_quality = app.get_config().read().prefer_audio_quality;
        let prefer_quality_id: i64 = prefer_quality.into();
        let prefer_quality_found = medias.iter().any(|m| m.id == prefer_quality_id);
        let quality_filtered_medias: Vec<MediaForPrepare> = if prefer_quality_found {
            // 如果用户指定质量存在，则使用用户指定的质量
            medias
                .into_iter()
                .filter(|m| m.id == prefer_quality_id)
                .collect()
        } else {
            // 否则使用最高质量
            medias
                .into_iter()
                .filter(|m| m.id == best_quality_id)
                .collect()
        };

        let media = &quality_filtered_medias[0];

        self.audio_quality = media.id.into();

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
            "{filename}.m4a.com.lanyeeee.bilibili-video-downloader"
        ));
        let (audio_task, episode_title, ids_string) = {
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
                download_task: download_task.clone(),
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
                Ok(i) => download_task.update_progress(|p| p.audio_task.chunks[i].completed = true),
                Err(err) => {
                    let err_title = format!("{ids_string} `{episode_title}`音频的一个分片下载失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                }
            }
        }

        let download_completed = download_task
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
            download_task.update_progress(|p| p.video_task.mark_uncompleted());
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

        download_task.update_progress(|p| p.audio_task.completed = true);

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MediaForPrepare {
    pub id: i64,
    pub url_with_content_length: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AudioQualityForPrepare {
    Audio64K,
    Audio132K,
    Audio192K,
    AudioDolby,
    AudioHiRes,
}

trait ToAudioQualityForPrepare {
    fn to_audio_quality_for_prepare(self) -> Option<AudioQualityForPrepare>;
}

impl ToAudioQualityForPrepare for i64 {
    fn to_audio_quality_for_prepare(self) -> Option<AudioQualityForPrepare> {
        let audio_quality: AudioQuality = self.into();
        match audio_quality {
            AudioQuality::Audio64K => Some(AudioQualityForPrepare::Audio64K),
            AudioQuality::Audio132K => Some(AudioQualityForPrepare::Audio132K),
            AudioQuality::Audio192K => Some(AudioQualityForPrepare::Audio192K),
            AudioQuality::AudioDolby => Some(AudioQualityForPrepare::AudioDolby),
            AudioQuality::AudioHiRes => Some(AudioQualityForPrepare::AudioHiRes),
            AudioQuality::Unknown => None,
        }
    }
}
