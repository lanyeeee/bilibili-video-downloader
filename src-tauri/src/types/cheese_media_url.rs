use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CheeseMediaUrl {
    pub accept_format: String,
    pub code: i64,
    pub seek_param: String,
    pub is_preview: i64,
    pub fnval: i64,
    pub video_project: bool,
    pub play_view_business_info: Option<PlayViewBusinessInfo>,
    pub fnver: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub result: String,
    pub seek_type: String,
    pub from: String,
    pub video_codecid: i64,
    pub no_rexcode: i64,
    pub format: String,
    pub support_formats: Vec<SupportFormatInCheese>,
    pub message: String,
    pub accept_quality: Vec<i64>,
    pub quality: i64,
    pub timelength: i64,
    pub durls: Vec<DurlInCheese>,
    pub has_paid: bool,
    pub dash: Option<DashInCheese>,
    pub accept_description: Vec<String>,
    pub status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PlayViewBusinessInfo {
    pub user_status: UserStatusInCheeseUrl,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserStatusInCheeseUrl {
    pub watch_progress: WatchProgress,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct WatchProgress {
    pub current_watch_progress: i64,
    pub last_ep_id: i64,
    pub last_ep_index: String,
    pub last_time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SupportFormatInCheese {
    pub display_desc: String,
    pub superscript: String,
    pub need_login: bool,
    pub codecs: Vec<String>,
    pub format: String,
    pub description: String,
    pub quality: i64,
    pub new_description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DashInCheese {
    pub duration: u64,
    pub min_buffer_time: f64,
    pub video: Vec<MediaInCheese>,
    pub audio: Option<Vec<MediaInCheese>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MediaInCheese {
    pub start_with_sap: i64,
    pub bandwidth: i64,
    pub sar: String,
    pub codecs: String,
    pub base_url: String,
    pub backup_url: Vec<String>,
    pub segment_base: SegmentBaseInCheese,
    pub frame_rate: String,
    pub codecid: i64,
    pub size: i64,
    pub mime_type: String,
    pub width: i64,
    pub id: i64,
    pub height: i64,
    pub md5: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SegmentBaseInCheese {
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DurlInCheese {
    pub durl: Vec<DurlDetailInCheese>,
    pub quality: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DurlDetailInCheese {
    pub size: i64,
    pub ahead: String,
    pub length: i64,
    pub vhead: String,
    pub backup_url: Vec<String>,
    pub url: String,
    pub order: i64,
    pub md5: String,
}
