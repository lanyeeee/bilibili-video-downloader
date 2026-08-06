use serde::{Deserialize, Deserializer};

pub mod audio_quality;
pub mod available_media_formats;
pub mod bangumi_follow_info;
pub mod bangumi_info;
pub mod bangumi_media_url;
pub mod bangumi_media_url_v2;
pub mod cheese_info;
pub mod cheese_media_url;
pub mod codec_type;
pub mod create_download_task_params;
pub mod fav_folders;
pub mod fav_info;
pub mod get_available_media_formats_params;
pub mod get_bangumi_follow_info_params;
pub mod get_bangumi_info_params;
pub mod get_cheese_info_params;
pub mod get_fav_info_params;
pub mod get_history_info_params;
pub mod get_normal_info_params;
pub mod get_user_video_info_params;
pub mod history_info;
pub mod log_metadata;
pub mod normal_info;
pub mod normal_media_url;
pub mod player_info;
pub mod plugin_info;
pub mod qrcode_data;
pub mod qrcode_status;
pub mod restart_download_task_params;
pub mod search_params;
pub mod search_result;
pub mod skip_segments;
pub mod subtitle;
pub mod tags;
pub mod user_info;
pub mod user_video_info;
pub mod video_quality;
pub mod watch_later_info;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
