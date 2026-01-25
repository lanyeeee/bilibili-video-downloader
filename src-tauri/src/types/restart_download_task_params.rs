use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::{
    audio_quality::AudioQuality, codec_type::CodecType, video_quality::VideoQuality,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct RestartDownloadTaskParams {
    pub task_id: String,

    pub video_task_selected: bool,
    pub audio_task_selected: bool,
    pub merge_selected: bool,
    pub embed_chapter_selected: bool,
    pub embed_skip_selected: bool,
    pub subtitle_task_selected: bool,
    pub xml_danmaku_selected: bool,
    pub ass_danmaku_selected: bool,
    pub json_danmaku_selected: bool,
    pub cover_task_selected: bool,
    pub nfo_task_selected: bool,
    pub json_task_selected: bool,

    pub video_quality: VideoQuality,
    pub codec_type: CodecType,
    pub audio_quality: AudioQuality,
}
