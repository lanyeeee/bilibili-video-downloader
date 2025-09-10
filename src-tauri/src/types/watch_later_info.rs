use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WatchLaterInfo {
    pub count: i64,
    pub list: Vec<MediaInWatchLater>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct MediaInWatchLater {
    pub aid: i64,
    pub videos: i64,
    pub tid: i64,
    pub tname: String,
    pub copyright: i64,
    pub pic: String,
    pub title: String,
    pub pubdate: i64,
    pub ctime: i64,
    pub desc: String,
    pub state: i64,
    pub duration: i64,
    pub redirect_url: Option<String>,
    pub mission_id: Option<i64>,
    pub rights: RightsInWatchLater,
    pub owner: OwnerInWatchLater,
    pub stat: StatInWatchLater,
    pub dynamic: String,
    pub dimension: DimensionInWatchLater,
    pub short_link_v2: String,
    pub up_from_v2: Option<i64>,
    pub first_frame: Option<String>,
    pub pub_location: Option<String>,
    pub cover43: String,
    pub tidv2: i64,
    pub tnamev2: String,
    pub pid_v2: i64,
    pub pid_name_v2: String,
    pub page: PageInWatchLater,
    pub count: i64,
    pub cid: i64,
    pub progress: i64,
    pub add_at: i64,
    pub bvid: String,
    pub uri: String,
    pub enable_vt: i64,
    pub view_text_1: String,
    pub card_type: i64,
    pub left_icon_type: i64,
    pub left_text: String,
    pub right_icon_type: i64,
    pub right_text: String,
    pub arc_state: i64,
    pub pgc_label: String,
    pub show_up: bool,
    pub forbid_fav: bool,
    pub forbid_sort: bool,
    pub season_title: String,
    pub long_title: String,
    pub index_title: String,
    pub c_source: String,
    pub season_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct RightsInWatchLater {
    pub bp: i64,
    pub elec: i64,
    pub download: i64,
    pub movie: i64,
    pub pay: i64,
    pub hd5: i64,
    pub no_reprint: i64,
    pub autoplay: i64,
    pub ugc_pay: i64,
    pub is_cooperation: i64,
    pub ugc_pay_preview: i64,
    pub no_background: i64,
    pub arc_pay: i64,
    pub pay_free_watch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct OwnerInWatchLater {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct StatInWatchLater {
    pub aid: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub favorite: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub dislike: i64,
    pub vt: i64,
    pub vv: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct DimensionInWatchLater {
    pub width: i64,
    pub height: i64,
    pub rotate: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PageInWatchLater {
    pub cid: i64,
    pub page: i64,
    pub from: String,
    pub part: String,
    pub duration: i64,
    pub vid: String,
    pub weblink: String,
    pub dimension: DimensionInWatchLater,
    pub first_frame: Option<String>,
    pub ctime: i64,
}
