use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserVideoInfo {
    pub list: UserVideoList,
    pub page: PageInUserVideo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserVideoList {
    pub vlist: Vec<EpInUserVideo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EpInUserVideo {
    pub comment: i64,
    pub typeid: i64,
    pub play: i64,
    pub pic: String,
    pub subtitle: String,
    pub description: String,
    pub copyright: String,
    pub title: String,
    pub review: i64,
    pub author: String,
    pub mid: i64,
    pub created: i64,
    pub length: String,
    pub video_review: i64,
    pub aid: i64,
    pub bvid: String,
    pub hide_click: bool,
    pub is_pay: i64,
    pub is_union_video: i64,
    pub is_steins_gate: i64,
    pub is_live_playback: i64,
    pub is_lesson_video: i64,
    pub is_lesson_finished: i64,
    pub lesson_update_info: String,
    pub jump_url: String,
    pub meta: Option<MetaInUserVideo>,
    pub is_avoided: i64,
    pub season_id: i64,
    pub attribute: i64,
    pub is_charging_arc: bool,
    pub elec_arc_type: i64,
    pub elec_arc_badge: String,
    pub vt: i64,
    pub enable_vt: i64,
    pub vt_display: String,
    pub playback_position: i64,
    pub is_self_view: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MetaInUserVideo {
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub mid: i64,
    pub intro: String,
    pub sign_state: i64,
    pub attribute: i64,
    pub stat: StatInUserVideo,
    pub ep_count: i64,
    pub first_aid: Option<i64>,
    pub ptime: i64,
    pub ep_num: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StatInUserVideo {
    pub season_id: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub favorite: i64,
    pub coin: i64,
    pub share: i64,
    pub like: i64,
    pub mtime: i64,
    pub vt: i64,
    pub vv: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PageInUserVideo {
    pub pn: i64,
    pub ps: i64,
    pub count: i64,
}
