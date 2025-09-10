use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PlayerInfo {
    pub aid: i64,
    pub bvid: String,
    pub allow_bp: bool,
    pub no_share: bool,
    pub cid: i64,
    pub max_limit: i64,
    pub page_no: i64,
    pub has_next: bool,
    pub ip_info: IpInfo,
    pub login_mid: i64,
    pub login_mid_hash: String,
    pub is_owner: bool,
    pub name: String,
    pub permission: String,
    pub level_info: LevelInfoInPlayerInfo,
    pub vip: VipInPlayerInfo,
    pub answer_status: i64,
    pub block_time: i64,
    pub role: String,
    pub last_play_time: i64,
    pub last_play_cid: i64,
    pub now_time: i64,
    pub online_count: i64,
    pub need_login_subtitle: bool,
    pub subtitle: SubtitleInPlayerInfo,
    pub view_points: Vec<ViewPoint>,
    pub preview_toast: String,
    pub options: Options,
    pub online_switch: OnlineSwitch,
    pub fawkes: Fawkes,
    pub show_switch: ShowSwitch,
    pub toast_block: bool,
    pub is_upower_exclusive: bool,
    pub is_upower_play: bool,
    pub is_ugc_pay_preview: bool,
    pub elec_high_level: ElecHighLevel,
    pub disable_show_up_info: bool,
    pub is_upower_exclusive_with_qa: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct IpInfo {
    pub ip: String,
    pub zone_ip: String,
    pub zone_id: i64,
    pub country: String,
    pub province: String,
    pub city: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct LevelInfoInPlayerInfo {
    pub current_level: i64,
    pub current_min: i64,
    pub current_exp: i64,
    pub next_exp: i64,
    pub level_up: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct VipInPlayerInfo {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub status: i64,
    pub due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
    pub label: LabelInPlayerInfo,
    pub avatar_subscript: i64,
    pub nickname_color: String,
    pub role: i64,
    pub avatar_subscript_url: String,
    pub tv_vip_status: i64,
    pub tv_vip_pay_type: i64,
    pub tv_due_date: i64,
    pub avatar_icon: AvatarIcon,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct LabelInPlayerInfo {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
    pub use_img_label: bool,
    pub img_label_uri_hans: String,
    pub img_label_uri_hant: String,
    pub img_label_uri_hans_static: String,
    pub img_label_uri_hant_static: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct AvatarIcon {
    pub icon_resource: IconResource,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct IconResource {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SubtitleInPlayerInfo {
    pub allow_submit: bool,
    pub lan: String,
    pub lan_doc: String,
    pub subtitles: Vec<SubtitleDetailInPlayerInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SubtitleDetailInPlayerInfo {
    pub id: i64,
    pub lan: String,
    pub lan_doc: String,
    pub is_lock: bool,
    pub subtitle_url: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub id_str: String,
    pub ai_type: i64,
    pub ai_status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ViewPoint {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub from: i64,
    pub to: i64,
    pub content: String,
    pub img_url: Option<String>,
    pub logo_url: Option<String>,
    pub team_type: String,
    pub team_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Options {
    pub is_360: bool,
    pub without_vip: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct OnlineSwitch {
    pub enable_gray_dash_playback: String,
    pub new_broadcast: String,
    pub realtime_dm: String,
    pub subtitle_submit_switch: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Fawkes {
    pub config_version: i64,
    pub ff_version: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ShowSwitch {
    pub long_progress: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ElecHighLevel {
    pub privilege_type: i64,
    pub title: String,
    pub sub_title: String,
    pub show_button: bool,
    pub button_text: String,
    pub jump_url: String,
    pub intro: String,
    pub new: bool,
    pub question_text: String,
    pub qa_title: String,
}
