use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
    get_fav_info_params::GetFavInfoParams, get_normal_info_params::GetNormalInfoParams,
    get_user_video_info_params::GetUserVideoInfoParams,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchParams {
    Normal(GetNormalInfoParams),
    Bangumi(GetBangumiInfoParams),
    Cheese(GetCheeseInfoParams),
    UserVideo(GetUserVideoInfoParams),
    Fav(GetFavInfoParams),
}
