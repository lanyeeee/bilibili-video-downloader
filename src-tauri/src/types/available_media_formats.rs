use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::{
    audio_quality::AudioQuality, codec_type::CodecType, video_quality::VideoQuality,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AvailableMediaFormats {
    pub video_qualities_and_codec_types: Vec<VideoQualityAndCodecType>,
    pub audio_qualities: Vec<AudioQuality>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct VideoQualityAndCodecType {
    pub video_quality: VideoQuality,
    pub codec_type: CodecType,
}
