use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::{
    danmaku_xml_to_ass::canvas::CanvasConfig,
    types::{audio_quality::AudioQuality, codec_type::CodecType, video_quality::VideoQuality},
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
#[allow(clippy::struct_field_names)]
pub struct Config {
    pub download_dir: PathBuf,
    pub enable_file_logger: bool,
    pub sessdata: String,
    pub video_quality_priority: Vec<VideoQuality>,
    pub codec_type_priority: Vec<CodecType>,
    pub audio_quality_priority: Vec<AudioQuality>,
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
    pub proxy_mode: ProxyMode,
    pub proxy_host: String,
    pub proxy_port: u16,
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
        let default_video_quality_priority = vec![
            VideoQuality::Video8K,
            VideoQuality::VideoDolby,
            VideoQuality::VideoHDR,
            VideoQuality::Video4K,
            VideoQuality::Video1080P60,
            VideoQuality::Video1080PPlus,
            VideoQuality::Video1080P,
            VideoQuality::VideoAiRepair,
            VideoQuality::Video720P60,
            VideoQuality::Video720P,
            VideoQuality::Video480P,
            VideoQuality::Video360P,
            VideoQuality::Video240P,
        ];
        let default_audio_quality_priority = vec![
            AudioQuality::AudioHiRes,
            AudioQuality::AudioDolby,
            AudioQuality::Audio192K,
            AudioQuality::Audio132K,
            AudioQuality::Audio64K,
        ];

        Config {
            download_dir: app_data_dir.join("视频下载"),
            enable_file_logger: true,
            sessdata: String::new(),
            video_quality_priority: default_video_quality_priority,
            codec_type_priority: vec![CodecType::AVC, CodecType::HEVC, CodecType::AV1],
            audio_quality_priority: default_audio_quality_priority,
            download_video: true,
            download_audio: true,
            auto_merge: true,
            embed_chapter: true,
            embed_skip: true,
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
            proxy_mode: ProxyMode::NoProxy,
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 7890,
            task_concurrency: 3,
            task_download_interval_sec: 0,
            chunk_concurrency: 16,
            chunk_download_interval_sec: 0,
            danmaku_config: CanvasConfig::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub enum ProxyMode {
    #[default]
    NoProxy,
    System,
    Custom,
}
