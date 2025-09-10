use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct QrcodeStatus {
    pub url: String,
    pub refresh_token: String,
    pub timestamp: i64,
    pub code: i64,
    pub message: String,
}
