use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum GetAvailableMediaFormatsParams {
    Normal(GetNormalAvailableMediaFormatsParams),
    Bangumi(GetBangumiAvailableMediaFormatsParams),
    Cheese(GetCheeseAvailableMediaFormatsParams),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetNormalAvailableMediaFormatsParams {
    pub bvid: String,
    pub cid: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetBangumiAvailableMediaFormatsParams {
    pub cid: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetCheeseAvailableMediaFormatsParams {
    pub ep_id: i64,
}
