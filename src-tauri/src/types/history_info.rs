use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct HistoryInfo {
    pub has_more: bool,
    pub page: PageInHistory,
    pub list: Option<Vec<HistoryDetail>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PageInHistory {
    pub pn: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct HistoryDetail {
    pub title: String,
    pub long_title: String,
    pub cover: String,
    pub uri: String,
    pub history: History,
    pub videos: i64,
    pub author_name: String,
    pub author_face: String,
    pub author_mid: i64,
    pub view_at: i64,
    pub progress: i64,
    pub badge: String,
    pub show_title: String,
    pub duration: i64,
    pub total: i64,
    pub new_desc: String,
    pub is_finish: i64,
    pub is_fav: i64,
    pub kid: i64,
    pub tag_name: String,
    pub live_status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct History {
    pub oid: i64,
    pub epid: i64,
    pub bvid: String,
    pub page: i64,
    pub cid: i64,
    pub part: String,
    pub business: String,
    pub dt: i64,
}
