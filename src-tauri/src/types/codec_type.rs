use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    IntoPrimitive,
    FromPrimitive,
)]
#[repr(i64)]
#[allow(clippy::upper_case_acronyms)]
pub enum CodecType {
    #[default]
    Unknown = -1,
    Audio = 0,
    AVC = 7,
    HEVC = 12,
    AV1 = 13,
}
