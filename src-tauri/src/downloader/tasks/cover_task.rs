use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CoverTask {
    pub selected: bool,
    pub url: String,
    pub completed: bool,
}

impl CoverTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }
}
