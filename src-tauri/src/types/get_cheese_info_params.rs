use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum GetCheeseInfoParams {
    EpId(i64),
    SeasonId(i64),
}
