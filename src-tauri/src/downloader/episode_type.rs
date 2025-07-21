use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub enum EpisodeType {
    #[default]
    Normal,
    Bangumi,
    Cheese,
}
