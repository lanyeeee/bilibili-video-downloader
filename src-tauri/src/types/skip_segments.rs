use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SkipSegments(pub Vec<SkipSegment>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SkipSegment {
    pub cid: String,
    pub category: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    pub segment: Vec<f64>,
    #[serde(rename = "UUID")]
    pub uuid: String,
    #[serde(rename = "videoDuration")]
    pub video_duration: i64,
    pub locked: i64,
    pub votes: i64,
    pub description: String,
}
