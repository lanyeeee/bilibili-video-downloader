use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::{
    audio_quality::AudioQuality,
    available_media_formats::{AvailableMediaFormats, VideoQualityAndCodecType},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NormalMediaUrl {
    pub from: String,
    pub result: String,
    pub message: String,
    pub quality: i64,
    pub format: String,
    pub timelength: i64,
    pub accept_format: String,
    pub accept_description: Vec<String>,
    pub accept_quality: Vec<i64>,
    pub video_codecid: i64,
    pub seek_param: String,
    pub seek_type: String,
    pub durl: Vec<DurlInNormal>,
    pub dash: DashInNormal,
    pub support_formats: Vec<SupportFormatInNormal>,
    pub last_play_time: i64,
    pub last_play_cid: i64,
    pub play_conf: PlayConf,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct DashInNormal {
    pub duration: u64,
    pub min_buffer_time: f64,
    pub video: Vec<MediaInNormal>,
    pub audio: Option<Vec<MediaInNormal>>,
    pub dolby: Dolby,
    pub flac: Option<Flac>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct DurlInNormal {
    pub order: i64,
    pub length: i64,
    pub size: i64,
    pub ahead: String,
    pub vhead: String,
    pub url: String,
    pub backup_url: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Flac {
    pub display: bool,
    pub audio: Option<MediaInNormal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct MediaInNormal {
    pub id: i64,
    pub start_with_sap: i64,
    pub bandwidth: i64,
    pub sar: String,
    pub codecs: String,
    pub base_url: String,
    pub backup_url: Vec<String>,
    pub segment_base: SegmentBaseInNormal,
    pub mime_type: String,
    pub frame_rate: String,
    pub width: i64,
    pub height: i64,
    pub codecid: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SegmentBaseInNormal {
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Dolby {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub audio: Option<Vec<MediaInNormal>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SupportFormatInNormal {
    pub quality: i64,
    pub format: String,
    pub new_description: String,
    pub display_desc: String,
    pub superscript: String,
    pub codecs: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PlayConf {
    pub is_new_description: bool,
}

impl NormalMediaUrl {
    pub fn to_get_available_media_formats_result(&self) -> AvailableMediaFormats {
        let mut video_qualities_and_codec_types: Vec<VideoQualityAndCodecType> = Vec::new();
        let mut audio_qualities: Vec<AudioQuality> = Vec::new();

        for media in &self.dash.video {
            let video_qualities_and_codec = VideoQualityAndCodecType {
                video_quality: media.id.into(),
                codec_type: media.codecid.into(),
            };

            video_qualities_and_codec_types.push(video_qualities_and_codec);
        }

        if !self.durl.is_empty() {
            let video_qualities_and_codec = VideoQualityAndCodecType {
                video_quality: self.quality.into(),
                codec_type: self.video_codecid.into(),
            };

            video_qualities_and_codec_types.push(video_qualities_and_codec);
        }

        if let Some(medias) = &self.dash.audio {
            for media in medias {
                audio_qualities.push(media.id.into());
            }
        }

        if let Some(medias) = &self.dash.dolby.audio {
            for media in medias {
                audio_qualities.push(media.id.into());
            }
        }

        let flac = self.dash.flac.as_ref();
        if let Some(media) = flac.and_then(|flac| flac.audio.as_ref()) {
            audio_qualities.push(media.id.into());
        }

        AvailableMediaFormats {
            video_qualities_and_codec_types,
            audio_qualities,
        }
    }
}
