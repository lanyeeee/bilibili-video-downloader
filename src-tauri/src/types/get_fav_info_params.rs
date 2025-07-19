use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetFavInfoParams {
    pub media_list_id: i64,
    pub pn: i64,
}
