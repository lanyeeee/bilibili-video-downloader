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
        bangumi_media_url::BangumiMediaUrl, cheese_media_url::CheeseMediaUrl,
        codec_type::CodecType, normal_media_url::NormalMediaUrl, video_quality::VideoQuality,
    },
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

        if medias.is_empty() {
            return Err(anyhow!("获取视频地址失败"));
        }

        self.prepare(app, medias);

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

        if medias.is_empty() {
            return Err(anyhow!("获取视频地址失败"));
        }

        self.prepare(app, medias);

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

        if medias.is_empty() {
            return Err(anyhow!("获取视频地址失败"));
        }

        self.prepare(app, medias);

        Ok(())
    }

    fn prepare(&mut self, app: &AppHandle, mut medias: Vec<MediaForPrepare>) {
        medias.sort_by_key(|m| Reverse(m.id));
        let best_quality_id = medias[0].id;

        let (prefer_quality, prefer_codec_type) = {
            let config = app.get_config().inner().read();
            (config.prefer_video_quality, config.prefer_codec_type)
        };

        let prefer_quality_id: i64 = prefer_quality.into();
        let prefer_codec_id: i64 = prefer_codec_type.into();
        let prefer_quality_found = medias.iter().any(|m| m.id == prefer_quality_id);
        let mut quality_filtered_medias: Vec<MediaForPrepare> = if prefer_quality_found {
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
        // 按照 AVC > HEVC > AV1 的顺序排列
        quality_filtered_medias.sort_by_key(|m| m.codecid);

        let media = quality_filtered_medias
            .iter()
            .find(|m| m.codecid == prefer_codec_id)
            .unwrap_or(&quality_filtered_medias[0]);

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

struct MediaForPrepare {
    pub id: i64,
    pub url_with_content_length: Vec<(String, u64)>,
    pub codecid: i64,
}
