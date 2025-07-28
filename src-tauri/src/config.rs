use std::path::{Path, PathBuf};

use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::danmaku_xml_to_ass::canvas::CanvasConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
#[allow(clippy::struct_field_names)]
pub struct Config {
    pub download_dir: PathBuf,
    pub enable_file_logger: bool,
    pub sessdata: String,
    pub prefer_video_quality: PreferVideoQuality,
    pub prefer_codec_type: PreferCodecType,
    pub prefer_audio_quality: PreferAudioQuality,
    pub download_video: bool,
    pub download_audio: bool,
    pub auto_merge: bool,
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
    pub task_concurrency: usize,
    pub task_download_interval_sec: u64,
    pub chunk_concurrency: usize,
    pub chunk_download_interval_sec: u64,
    pub danmaku_config: CanvasConfig,
}

impl Config {
    pub fn new(app: &AppHandle) -> anyhow::Result<Config> {
        let app_data_dir = app.path().app_data_dir()?;
        let config_path = app_data_dir.join("config.json");

        let config = if config_path.exists() {
            let config_string = std::fs::read_to_string(config_path)?;
            match serde_json::from_str(&config_string) {
                // 如果能够直接解析为Config，则直接返回
                Ok(config) => config,
                // 否则，将默认配置与文件中已有的配置合并
                // 以免新版本添加了新的配置项，用户升级到新版本后，所有配置项都被重置
                Err(_) => Config::merge_config(&config_string, &app_data_dir),
            }
        } else {
            Config::default(&app_data_dir)
        };
        config.save(app)?;
        Ok(config)
    }

    pub fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let app_data_dir = app.path().app_data_dir()?;
        let config_path = app_data_dir.join("config.json");
        let config_string = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_string)?;
        Ok(())
    }

    fn merge_config(config_string: &str, app_data_dir: &Path) -> Config {
        let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(config_string) else {
            return Config::default(app_data_dir);
        };
        let serde_json::Value::Object(ref mut map) = json_value else {
            return Config::default(app_data_dir);
        };
        let Ok(default_config_value) = serde_json::to_value(Config::default(app_data_dir)) else {
            return Config::default(app_data_dir);
        };
        let serde_json::Value::Object(default_map) = default_config_value else {
            return Config::default(app_data_dir);
        };
        for (key, value) in default_map {
            map.entry(key).or_insert(value);
        }
        let Ok(config) = serde_json::from_value(json_value) else {
            return Config::default(app_data_dir);
        };
        config
    }

    fn default(app_data_dir: &Path) -> Config {
        const DEFAULT_FMT_FOR_PART: &str =
            "{collection_title}/{episode_title}/{episode_title}-P{part_order} {part_title}";
        Config {
            download_dir: app_data_dir.join("视频下载"),
            enable_file_logger: true,
            sessdata: String::new(),
            prefer_video_quality: PreferVideoQuality::Best,
            prefer_codec_type: PreferCodecType::AVC,
            prefer_audio_quality: PreferAudioQuality::Best,
            download_video: true,
            download_audio: true,
            auto_merge: true,
            download_xml_danmaku: true,
            download_ass_danmaku: true,
            download_json_danmaku: true,
            download_subtitle: true,
            download_cover: true,
            download_nfo: true,
            download_json: true,
            dir_fmt: "{collection_title}/{episode_title}".to_string(),
            dir_fmt_for_part: DEFAULT_FMT_FOR_PART.to_string(),
            time_fmt: "%Y-%m-%d_%H-%M-%S".to_string(),
            task_concurrency: 3,
            task_download_interval_sec: 0,
            chunk_concurrency: 16,
            chunk_download_interval_sec: 0,
            danmaku_config: CanvasConfig::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    IntoPrimitive,
    FromPrimitive,
)]
#[repr(i64)]
pub enum PreferVideoQuality {
    #[default]
    Best = -1,

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

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    IntoPrimitive,
    FromPrimitive,
)]
#[repr(i64)]
#[allow(clippy::upper_case_acronyms)]
pub enum PreferCodecType {
    #[default]
    Unknown = -1,

    AVC = 7,
    HEVC = 12,
    AV1 = 13,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    IntoPrimitive,
    FromPrimitive,
)]
#[repr(i64)]
pub enum PreferAudioQuality {
    #[default]
    Best = -1,
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
