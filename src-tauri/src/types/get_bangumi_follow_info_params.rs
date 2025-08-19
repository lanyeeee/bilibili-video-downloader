use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetBangumiFollowInfoParams {
    pub vmid: i64,
    /// 1: 番剧 2: 电视剧或电影
    #[serde(rename = "type")]
    pub type_field: i64,
    pub pn: i64,
    // 0: 全部 1: 想看 2: 在看 3: 看过
    pub follow_status: i64,
}
