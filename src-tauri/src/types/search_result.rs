use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    bangumi_info::{self, BangumiInfo},
    cheese_info::{self, CheeseInfo},
    fav_info::FavInfo,
    normal_info::NormalInfo,
    user_video_info::UserVideoInfo,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchResult {
    Normal(NormalSearchResult),
    Bangumi(BangumiSearchResult),
    Cheese(CheeseSearchResult),
    UserVideo(UserVideoSearchResult),
    Fav(FavSearchResult),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct NormalSearchResult(pub NormalInfo);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BangumiSearchResult {
    pub ep: Option<bangumi_info::EpInBangumi>,
    pub info: BangumiInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CheeseSearchResult {
    pub ep: Option<cheese_info::EpInCheese>,
    pub info: CheeseInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserVideoSearchResult(pub UserVideoInfo);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FavSearchResult(pub FavInfo);
