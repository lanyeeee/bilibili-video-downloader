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
    #[num_enum(alternatives = [100008])]
    Audio64K = 30216,
    #[serde(rename = "132K")]
    #[num_enum(alternatives = [100009])]
    Audio132K = 30232,
    #[serde(rename = "192K")]
    #[num_enum(alternatives = [100010])]
    Audio192K = 30280,
    #[serde(rename = "Dolby")]
    AudioDolby = 30250,
    #[serde(rename = "HiRes")]
    AudioHiRes = 30251,
}
