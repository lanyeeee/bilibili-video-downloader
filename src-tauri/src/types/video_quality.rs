use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    IntoPrimitive,
    FromPrimitive,
)]
#[repr(i64)]
pub enum VideoQuality {
    #[default]
    Unknown = -1,

    #[serde(rename = "240P")]
    Video240P = 6,
    #[serde(rename = "360P")]
    Video360P = 16,
    #[serde(rename = "480P")]
    Video480P = 32,
    #[serde(rename = "720P")]
    Video720P = 64,
    #[serde(rename = "720P60")]
    Video720P60 = 74,
    #[serde(rename = "1080P")]
    Video1080P = 80,
    #[serde(rename = "AiRepair")]
    VideoAiRepair = 100,
    #[serde(rename = "1080P+")]
    Video1080PPlus = 112,
    #[serde(rename = "1080P60")]
    Video1080P60 = 116,
    #[serde(rename = "4K")]
    Video4K = 120,
    #[serde(rename = "HDR")]
    VideoHDR = 125,
    #[serde(rename = "Dolby")]
    VideoDolby = 126,
    #[serde(rename = "8K")]
    Video8K = 127,
}
