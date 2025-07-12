use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum GetNormalInfoParams {
    Bvid(String),
    Aid(i64),
}
