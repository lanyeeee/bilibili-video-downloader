use serde::{Deserialize, Serialize};
use specta::Type;

use super::{bangumi_info::BangumiInfo, cheese_info::CheeseInfo, normal_info::NormalInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum CreateDownloadTaskParams {
    Normal(CreateNormalDownloadTaskParams),
    Bangumi(CreateBangumiDownloadTaskParams),
    Cheese(CreateCheeseDownloadTaskParams),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CreateNormalDownloadTaskParams {
    pub info: NormalInfo,
    pub aid_cid_pairs: Vec<(i64, Option<i64>)>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CreateBangumiDownloadTaskParams {
    pub ep_ids: Vec<i64>,
    pub info: BangumiInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CreateCheeseDownloadTaskParams {
    pub ep_ids: Vec<i64>,
    pub info: CheeseInfo,
}
