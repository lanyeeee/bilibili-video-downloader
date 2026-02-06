use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::bangumi_media_url::BangumiMediaUrl;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct BangumiMediaUrlV2 {
    pub play_view_business_info: PlayViewBusinessInfo,
    pub video_info: BangumiMediaUrl,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PlayViewBusinessInfo {
    pub episode_info: EpisodeInfoInBangumi,
    pub season_info: SeasonInfoInBangumi,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct EpisodeInfoInBangumi {
    pub aid: i64,
    pub bvid: String,
    pub cid: i64,
    pub delivery_business_fragment_video: bool,
    pub delivery_fragment_video: bool,
    pub ep_id: i64,
    pub ep_status: i64,
    pub interaction: Interaction,
    pub long_title: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Interaction {
    pub interaction: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SeasonInfoInBangumi {
    pub season_id: i64,
    pub season_type: i64,
}
