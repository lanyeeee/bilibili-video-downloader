use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
pub struct DanmakuTask {
    pub xml_selected: bool,
    pub ass_selected: bool,
    pub json_selected: bool,
    pub completed: bool,
}

impl DanmakuTask {
    pub fn is_completed(&self) -> bool {
        !self.xml_selected && !self.ass_selected && !self.json_selected || self.completed
    }
}
