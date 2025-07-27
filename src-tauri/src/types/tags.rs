use serde::{Deserialize, Serialize};
use specta::Type;

pub type Tags = Vec<Tag>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct Tag {
    pub tag_id: i64,
    pub tag_name: String,
    pub music_id: String,
    pub tag_type: String,
    pub jump_url: String,
}
