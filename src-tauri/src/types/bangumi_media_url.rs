use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BangumiMediaUrl {
    pub accept_format: String,
    pub code: i64,
    pub seek_param: String,
    pub is_preview: i64,
    pub fnval: i64,
    pub video_project: bool,
    pub fnver: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub bp: i64,
    pub seek_type: String,
    pub result: String,
    pub vip_type: Option<i64>,
    pub from: String,
    pub video_codecid: i64,
    pub record_info: Option<RecordInfo>,
    pub is_drm: bool,
    pub no_rexcode: i64,
    pub format: String,
    pub support_formats: Vec<SupportFormatInBangumi>,
    pub message: String,
    pub accept_quality: Vec<i64>,
    pub quality: i64,
    pub timelength: i64,
    pub durls: Vec<DurlInBangumi>,
    pub has_paid: bool,
    pub vip_status: Option<i64>,
    pub error_code: i64,
    pub dash: Option<DashInBangumi>,
    pub clip_info_list: Vec<ClipInfoList>,
    pub accept_description: Vec<String>,
    pub status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RecordInfo {
    pub record_icon: String,
    pub record: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SupportFormatInBangumi {
    pub display_desc: String,
    pub has_preview: bool,
    pub sub_description: String,
    pub superscript: String,
    pub need_login: Option<bool>,
    pub codecs: Vec<String>,
    pub format: String,
    pub description: String,
    pub need_vip: Option<bool>,
    pub attribute: i64,
    pub quality: i64,
    pub new_description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DashInBangumi {
    pub duration: u64,
    pub min_buffer_time: f64,
    pub video: Vec<MediaInBangumi>,
    pub audio: Option<Vec<MediaInBangumi>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MediaInBangumi {
    pub start_with_sap: i64,
    pub bandwidth: i64,
    pub sar: String,
    pub backup_url: Vec<String>,
    pub codecs: String,
    pub base_url: String,
    pub segment_base: SegmentBaseInBangumi,
    pub mime_type: String,
    pub frame_rate: String,
    pub codecid: i64,
    pub size: i64,
    pub width: i64,
    pub id: i64,
    pub height: i64,
    pub md5: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SegmentBaseInBangumi {
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ClipInfoList {
    #[serde(rename = "materialNo")]
    pub material_no: i64,
    pub start: i64,
    pub end: i64,
    #[serde(rename = "toastText")]
    pub toast_text: String,
    #[serde(rename = "clipType")]
    pub clip_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DurlInBangumi {
    pub durl: Vec<DurlDetailInBangumi>,
    pub quality: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DurlDetailInBangumi {
    pub size: i64,
    pub ahead: String,
    pub length: i64,
    pub vhead: String,
    pub backup_url: Vec<String>,
    pub url: String,
    pub order: i64,
    pub md5: String,
}
