use serde::{Deserialize, Serialize};
use specta::Type;

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
    pub codecs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PlayConf {
    pub is_new_description: bool,
}
