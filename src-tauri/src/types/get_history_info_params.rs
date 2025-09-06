use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetHistoryInfoParams {
    pub pn: i64,
    pub keyword: String,
    pub add_time_start: i64,
    pub add_time_end: i64,
    pub arc_max_duration: i64,
    pub arc_min_duration: i64,
    pub device_type: DeviceType,
}

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
pub enum DeviceType {
    #[default]
    All = 0,
    PC = 1,
    Mobile = 2,
    Pad = 3,
    TV = 4,
}
