use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetUserVideoInfoParams {
    pub pn: i64,
    pub mid: i64,
}
