use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SubtitleTask {
    pub selected: bool,
    pub completed: bool,
}

impl SubtitleTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }
}
