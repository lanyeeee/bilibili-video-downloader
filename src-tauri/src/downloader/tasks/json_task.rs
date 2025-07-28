use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct JsonTask {
    pub selected: bool,
    pub completed: bool,
}

impl JsonTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }
}
