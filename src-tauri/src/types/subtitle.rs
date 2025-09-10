use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Subtitle {
    pub font_size: f64,
    pub font_color: String,
    pub background_alpha: f64,
    pub background_color: String,
    #[serde(rename = "Stroke")]
    pub stroke: String,
    pub body: Vec<Body>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Body {
    pub from: f64,
    pub to: f64,
    pub location: i64,
    pub content: String,
}
