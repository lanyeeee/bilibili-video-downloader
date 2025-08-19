use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BangumiFollowInfo {
    pub list: Vec<EpInBangumiFollow>,
    pub pn: i64,
    pub ps: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EpInBangumiFollow {
    pub season_id: i64,
    pub media_id: i64,
    pub season_type: i64,
    pub season_type_name: String,
    pub title: String,
    pub cover: String,
    pub total_count: i64,
    pub is_finish: i64,
    pub is_started: i64,
    pub is_play: i64,
    pub badge: String,
    pub badge_type: i64,
    pub rights: RightsInBangumiFollow,
    pub stat: StatInBangumiFollow,
    pub new_ep: NewEpInBangumiFollow,
    pub rating: Option<RatingInBangumiFollow>,
    pub square_cover: String,
    pub season_status: i64,
    pub season_title: String,
    pub badge_ep: String,
    pub media_attr: i64,
    pub season_attr: i64,
    pub evaluate: String,
    pub areas: Vec<AreaInBangumiFollow>,
    pub subtitle: String,
    pub first_ep: i64,
    pub can_watch: i64,
    pub release_date_show: Option<String>,
    pub series: SeriesInBangumiFollow,
    pub publish: PublishInBangumiFollow,
    pub mode: i64,
    pub section: Vec<SectionInBangumiFollow>,
    pub url: String,
    pub badge_info: BadgeInfoInBangumiFollow,
    pub renewal_time: Option<String>,
    pub first_ep_info: FirstEpInfo,
    pub formal_ep_count: Option<i64>,
    pub short_url: String,
    pub badge_infos: Option<BadgeInfos>,
    pub season_version: Option<String>,
    pub horizontal_cover_16_9: Option<String>,
    pub horizontal_cover_16_10: Option<String>,
    pub subtitle_14: Option<String>,
    pub viewable_crowd_type: i64,
    #[serde(default)]
    pub producers: Vec<Producer>,
    pub summary: String,
    #[serde(default)]
    pub styles: Vec<String>,
    pub follow_status: i64,
    pub is_new: i64,
    pub progress: String,
    pub both_follow: bool,
    pub subtitle_25: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RightsInBangumiFollow {
    pub allow_review: Option<i64>,
    pub allow_preview: Option<i64>,
    pub is_selection: i64,
    pub selection_style: i64,
    pub is_rcmd: Option<i64>,
    pub allow_bp_rank: Option<i64>,
    pub allow_bp: Option<i64>,
    pub allow_download: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StatInBangumiFollow {
    pub follow: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub coin: i64,
    pub series_follow: Option<i64>,
    pub series_view: Option<i64>,
    pub likes: i64,
    pub favorite: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct NewEpInBangumiFollow {
    pub id: Option<i64>,
    pub index_show: Option<String>,
    pub cover: Option<String>,
    pub title: Option<String>,
    pub long_title: Option<String>,
    pub pub_time: Option<String>,
    pub duration: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RatingInBangumiFollow {
    pub score: f64,
    pub count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AreaInBangumiFollow {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SeriesInBangumiFollow {
    pub series_id: Option<i64>,
    pub title: Option<String>,
    pub season_count: Option<i64>,
    pub new_season_id: Option<i64>,
    pub series_ord: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PublishInBangumiFollow {
    pub pub_time: String,
    pub pub_time_show: String,
    pub release_date: String,
    pub release_date_show: String,
    pub pub_time_show_db: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SectionInBangumiFollow {
    pub section_id: i64,
    pub season_id: i64,
    pub limit_group: i64,
    pub watch_platform: i64,
    pub copyright: String,
    pub ban_area_show: i64,
    pub episode_ids: Vec<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<i64>,
    pub title: Option<String>,
    pub attr: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BadgeInfoInBangumiFollow {
    pub text: Option<String>,
    pub bg_color: String,
    pub bg_color_night: String,
    pub img: Option<String>,
    pub multi_img: MultiImg,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MultiImg {
    pub color: String,
    pub medium_remind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FirstEpInfo {
    pub id: i64,
    pub cover: String,
    pub title: String,
    pub long_title: Option<String>,
    pub pub_time: String,
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BadgeInfos {
    pub vip_or_pay: Option<VipOrPay>,
    pub content_attr: Option<ContentAttr>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct VipOrPay {
    pub text: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub img: String,
    pub multi_img: MultiImg,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ContentAttr {
    pub text: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub img: String,
    pub multi_img: MultiImg,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Producer {
    pub mid: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub is_contribute: Option<i64>,
    pub title: String,
}
