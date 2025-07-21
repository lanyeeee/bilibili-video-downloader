use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MediaChunk {
    pub start: u64,
    pub end: u64,
    pub completed: bool,
}
