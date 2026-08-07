pub const SDK_API_VERSION_V1: u32 = 1;

pub mod v1 {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum HookPointV1 {
        AfterPrepare,
        BeforeVideoProcess,
        OnCompleted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PluginFailurePolicy {
        FailOpen,
        FailClosed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PluginDescriptorV1 {
        pub sdk_api_version: u32,
        pub id: String,
        pub name: String,
        pub version: String,
        pub hooks: Vec<HookPointV1>,
        pub failure_policy: PluginFailurePolicy,
        pub description: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct HookReadonlyMetaV1 {
        pub app_version: String,
        pub os: String,
        pub arch: String,
        pub process_id: u32,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BeforeVideoProcessPayloadV1 {
        pub progress: DownloadProgressV1,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AfterPreparePayloadV1 {
        pub progress: DownloadProgressV1,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OnCompletedPayloadV1 {
        pub progress: DownloadProgressV1,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum HookPayloadV1 {
        BeforeVideoProcess(BeforeVideoProcessPayloadV1),
        AfterPrepare(AfterPreparePayloadV1),
        OnCompleted(OnCompletedPayloadV1),
    }

    impl Default for HookPayloadV1 {
        fn default() -> Self {
            Self::BeforeVideoProcess(BeforeVideoProcessPayloadV1::default())
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct HookInputV1 {
        pub hook_point: HookPointV1,
        pub payload: HookPayloadV1,
        pub readonly_meta: HookReadonlyMetaV1,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct HookOutputV1 {
        pub payload: HookPayloadV1,
    }

    pub type HostApiGetConfigJsonV1 =
        unsafe extern "C" fn(out_ptr: *mut *mut u8, out_len: *mut usize) -> i32;
    pub type HostApiFreeBufferV1 = unsafe extern "C" fn(ptr: *mut u8, len: usize);

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct HostApiV1 {
        pub get_config_json: HostApiGetConfigJsonV1,
        pub free_buffer: HostApiFreeBufferV1,
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ProxyModeV1 {
        #[default]
        NoProxy,
        System,
        Custom,
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum FileExistActionV1 {
        #[default]
        Overwrite,
        Skip,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct CanvasConfigV1 {
        pub duration: f64,
        pub width: u32,
        pub height: u32,
        pub font: String,
        pub font_size: u32,
        pub width_ratio: f64,
        pub horizontal_gap: f64,
        pub lane_size: u32,
        pub float_percentage: f64,
        pub alpha: f64,
        pub bold: bool,
        pub outline: f64,
        pub time_offset: f64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    #[allow(clippy::struct_excessive_bools)]
    #[allow(clippy::struct_field_names)]
    pub struct HostConfigV1 {
        pub download_dir: PathBuf,
        pub enable_file_logger: bool,
        pub sessdata: String,
        pub video_quality_priority: Vec<VideoQualityV1>,
        pub codec_type_priority: Vec<CodecTypeV1>,
        pub audio_quality_priority: Vec<AudioQualityV1>,
        pub download_video: bool,
        pub download_audio: bool,
        pub auto_merge: bool,
        pub embed_chapter: bool,
        pub embed_skip: bool,
        pub download_xml_danmaku: bool,
        pub download_ass_danmaku: bool,
        pub download_json_danmaku: bool,
        pub download_subtitle: bool,
        pub download_cover: bool,
        pub download_nfo: bool,
        pub download_json: bool,
        pub dir_fmt: String,
        pub dir_fmt_for_part: String,
        pub time_fmt: String,
        pub proxy_mode: ProxyModeV1,
        pub proxy_host: String,
        pub proxy_port: u16,
        pub task_concurrency: usize,
        pub task_download_interval_sec: u64,
        pub chunk_concurrency: usize,
        pub chunk_download_interval_sec: u64,
        pub danmaku_config: CanvasConfigV1,
        pub file_exist_action: FileExistActionV1,
        pub auto_start_download_task: bool,
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EpisodeTypeV1 {
        #[default]
        Normal,
        Bangumi,
        Cheese,
    }

    #[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
    #[repr(i64)]
    pub enum VideoQualityV1 {
        #[default]
        Unknown = -1,
        #[serde(rename = "240P")]
        Video240P = 6,
        #[serde(rename = "360P")]
        Video360P = 16,
        #[serde(rename = "480P")]
        Video480P = 32,
        #[serde(rename = "720P")]
        Video720P = 64,
        #[serde(rename = "720P60")]
        Video720P60 = 74,
        #[serde(rename = "1080P")]
        Video1080P = 80,
        #[serde(rename = "AiRepair")]
        VideoAiRepair = 100,
        #[serde(rename = "1080P+")]
        Video1080PPlus = 112,
        #[serde(rename = "1080P60")]
        Video1080P60 = 116,
        #[serde(rename = "4K")]
        Video4K = 120,
        #[serde(rename = "HDR")]
        VideoHDR = 125,
        #[serde(rename = "Dolby")]
        VideoDolby = 126,
        #[serde(rename = "8K")]
        Video8K = 127,
    }

    #[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
    #[repr(i64)]
    pub enum AudioQualityV1 {
        #[default]
        Unknown = -1,
        #[serde(rename = "64K")]
        Audio64K = 30216,
        #[serde(rename = "132K")]
        Audio132K = 30232,
        #[serde(rename = "192K")]
        Audio192K = 30280,
        #[serde(rename = "Dolby")]
        AudioDolby = 30250,
        #[serde(rename = "HiRes")]
        AudioHiRes = 30251,
    }

    #[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
    #[repr(i64)]
    pub enum CodecTypeV1 {
        #[default]
        Unknown = -1,
        Audio = 0,
        AVC = 7,
        HEVC = 12,
        AV1 = 13,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct MediaChunkV1 {
        pub start: u64,
        pub end: u64,
        pub completed: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct VideoTaskV1 {
        pub selected: bool,
        pub url: String,
        pub video_quality: VideoQualityV1,
        pub codec_type: CodecTypeV1,
        pub content_length: u64,
        pub chunks: Vec<MediaChunkV1>,
        pub completed: bool,
        pub skipped: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct AudioTaskV1 {
        pub selected: bool,
        pub url: String,
        pub audio_quality: AudioQualityV1,
        pub content_length: u64,
        pub chunks: Vec<MediaChunkV1>,
        pub completed: bool,
        pub skipped: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    #[allow(clippy::struct_excessive_bools)]
    pub struct VideoProcessTaskV1 {
        pub merge_selected: bool,
        pub embed_chapter_selected: bool,
        pub embed_skip_selected: bool,
        pub completed: bool,
        pub skipped: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct SubtitleTaskV1 {
        pub selected: bool,
        pub completed: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    #[allow(clippy::struct_excessive_bools)]
    pub struct DanmakuTaskV1 {
        pub xml_selected: bool,
        pub ass_selected: bool,
        pub json_selected: bool,
        pub completed: bool,
        pub skipped: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct CoverTaskV1 {
        pub selected: bool,
        pub url: String,
        pub completed: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct NfoTaskV1 {
        pub selected: bool,
        pub completed: bool,
        pub skipped: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct JsonTaskV1 {
        pub selected: bool,
        pub completed: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct DownloadProgressV1 {
        pub task_id: String,
        pub episode_type: EpisodeTypeV1,
        pub aid: i64,
        pub bvid: Option<String>,
        pub cid: i64,
        pub ep_id: Option<i64>,
        pub duration: u64,
        pub pub_ts: i64,
        pub collection_title: String,
        pub part_title: Option<String>,
        pub part_order: Option<i64>,
        pub episode_title: String,
        pub episode_title_in_collection: Option<String>,
        pub episode_order: i64,
        pub up_name: Option<String>,
        pub up_uid: Option<i64>,
        pub up_avatar: Option<String>,
        pub episode_dir: PathBuf,
        pub filename: String,
        pub video_task: VideoTaskV1,
        pub audio_task: AudioTaskV1,
        pub video_process_task: VideoProcessTaskV1,
        pub subtitle_task: SubtitleTaskV1,
        pub danmaku_task: DanmakuTaskV1,
        pub cover_task: CoverTaskV1,
        pub nfo_task: NfoTaskV1,
        pub json_task: JsonTaskV1,
        pub create_ts: u64,
        pub completed_ts: Option<u64>,
        pub is_drm: bool,
        pub is_preview: bool,
    }
}
