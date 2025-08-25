use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EmbedChapterTask {
    pub selected: bool,
    pub completed: bool,
}

impl EmbedChapterTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }
}
