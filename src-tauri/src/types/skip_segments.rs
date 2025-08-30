use serde::{Deserialize, Serialize};
use specta::Type;

use crate::downloader::chapter_segments::ChapterSegment;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SkipSegments(pub Vec<SkipSegment>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SkipSegment {
    pub cid: String,
    pub category: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    pub segment: Vec<f64>,
    #[serde(rename = "UUID")]
    pub uuid: String,
    #[serde(rename = "videoDuration")]
    pub video_duration: f64,
    pub locked: i64,
    pub votes: i64,
    pub description: String,
}

impl SkipSegment {
    fn get_title(&self) -> Option<String> {
        match self.category.as_str() {
            "sponsor" => Some("广告".to_string()),
            "selfpromo" => Some("无偿/自我推广".to_string()),
            "exclusive_access" => Some("柔性推广/品牌合作".to_string()),
            "interaction" => Some("三连/订阅提醒".to_string()),
            "poi_highlight" => Some("精彩时刻/重点".to_string()),
            "intro" => Some("过场/开场动画".to_string()),
            "outro" => Some("鸣谢/结束画面".to_string()),
            "preview" => Some("回顾/概要".to_string()),
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn into_chapter_segment(self) -> Option<ChapterSegment> {
        if self.segment.len() < 2 {
            return None; // 确保 segment 包含开始和结束时间
        }

        Some(ChapterSegment {
            title: self.get_title()?,
            start: self.segment[0] as i64,
            end: self.segment[1] as i64,
        })
    }
}
