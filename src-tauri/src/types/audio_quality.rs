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
pub enum AudioQuality {
    #[default]
    Unknown = -1,

    #[serde(rename = "64K")]
    Audio64K = 30216,
    #[serde(rename = "132K")]
    Audio132K = 30232,
    #[serde(rename = "192K")]
    Audio192K = 30280,
    #[serde(rename = "Dolby")]
    AudioDolby = 30250,
    #[serde(rename = "HiRes")]
    AudioHiRes = 30251,
}
