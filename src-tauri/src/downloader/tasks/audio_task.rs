use std::cmp::Reverse;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tokio::task::JoinSet;

use crate::{
    downloader::media_chunk::MediaChunk,
    extensions::AppHandleExt,
    types::{
        audio_quality::AudioQuality, bangumi_media_url::BangumiMediaUrl,
        cheese_media_url::CheeseMediaUrl, normal_media_url::NormalMediaUrl,
    },
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
