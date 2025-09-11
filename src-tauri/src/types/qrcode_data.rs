use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize, Type)]
#[serde(default)]
pub struct QrcodeData {
    pub url: String,
    pub qrcode_key: String,
}
