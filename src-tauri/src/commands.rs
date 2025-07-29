use parking_lot::RwLock;
use tauri::AppHandle;

use crate::{
    config::Config,
    errors::{CommandError, CommandResult},
    extensions::AppHandleExt,
    logger,
    types::{
        bangumi_info::BangumiInfo, bangumi_media_url::BangumiMediaUrl, cheese_info::CheeseInfo,
        cheese_media_url::CheeseMediaUrl, create_download_task_params::CreateDownloadTaskParams,
        fav_folders::FavFolders, fav_info::FavInfo, get_bangumi_info_params::GetBangumiInfoParams,
        get_cheese_info_params::GetCheeseInfoParams, get_fav_info_params::GetFavInfoParams,
        get_normal_info_params::GetNormalInfoParams,
        get_user_video_info_params::GetUserVideoInfoParams, normal_info::NormalInfo,
        normal_media_url::NormalMediaUrl, player_info::PlayerInfo, qrcode_data::QrcodeData,
        qrcode_status::QrcodeStatus, user_info::UserInfo, user_video_info::UserVideoInfo,
        watch_later_info::WatchLaterInfo,
    },
};

#[tauri::command]
#[specta::specta]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config: tauri::State<RwLock<Config>>) -> Config {
    config.read().clone()
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn save_config(app: AppHandle, config: Config) -> CommandResult<()> {
    let config_state = app.get_config();

    let enable_file_logger = config.enable_file_logger;
    let enable_file_logger_changed = config_state
        .read()
        .enable_file_logger
        .ne(&enable_file_logger);

    {
        // 包裹在大括号中，以便自动释放写锁
        let mut config_state = config_state.write();
        *config_state = config;
        config_state
            .save(&app)
            .map_err(|err| CommandError::from("保存配置失败", err))?;
        tracing::debug!("保存配置成功");
    }

    if enable_file_logger_changed {
        if enable_file_logger {
            logger::reload_file_logger()
                .map_err(|err| CommandError::from("重新加载文件日志失败", err))?;
        } else {
            logger::disable_file_logger()
                .map_err(|err| CommandError::from("禁用文件日志失败", err))?;
        }
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub async fn generate_qrcode(app: AppHandle) -> CommandResult<QrcodeData> {
    let bili_client = app.get_bili_client();
    let qrcode_data = bili_client
        .generate_qrcode()
        .await
        .map_err(|err| CommandError::from("生成二维码失败", err))?;
    Ok(qrcode_data)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub async fn get_qrcode_status(app: AppHandle, qrcode_key: String) -> CommandResult<QrcodeStatus> {
    let bili_client = app.get_bili_client();
    let qrcode_status = bili_client
        .get_qrcode_status(&qrcode_key)
        .await
        .map_err(|err| CommandError::from("获取二维码状态", err))?;
    Ok(qrcode_status)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_user_info(app: AppHandle, sessdata: String) -> CommandResult<UserInfo> {
    let bili_client = app.get_bili_client();
    let user_info = bili_client
        .get_user_info(&sessdata)
        .await
        .map_err(|err| CommandError::from("获取用户信息失败", err))?;
    Ok(user_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_normal_info(
    app: AppHandle,
    params: GetNormalInfoParams,
) -> CommandResult<NormalInfo> {
    let bili_client = app.get_bili_client();
    let normal_info = bili_client
        .get_normal_info(params)
        .await
        .map_err(|err| CommandError::from("获取普通视频信息失败", err))?;
    Ok(normal_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_bangumi_info(
    app: AppHandle,
    params: GetBangumiInfoParams,
) -> CommandResult<BangumiInfo> {
    let bili_client = app.get_bili_client();
    let bangumi_info = bili_client
        .get_bangumi_info(params)
        .await
        .map_err(|err| CommandError::from("获取番剧视频信息失败", err))?;
    Ok(bangumi_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_cheese_info(
    app: AppHandle,
    params: GetCheeseInfoParams,
) -> CommandResult<CheeseInfo> {
    let bili_client = app.get_bili_client();
    let cheese_info = bili_client
        .get_cheese_info(params)
        .await
        .map_err(|err| CommandError::from("获取课程视频信息失败", err))?;
    Ok(cheese_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_user_video_info(
    app: AppHandle,
    params: GetUserVideoInfoParams,
) -> CommandResult<UserVideoInfo> {
    let bili_client = app.get_bili_client();
    let user_video_info = bili_client
        .get_user_video_info(params)
        .await
        .map_err(|err| CommandError::from("获取用户视频信息失败", err))?;
    Ok(user_video_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_normal_url(
    app: AppHandle,
    bvid: String,
    cid: i64,
) -> CommandResult<NormalMediaUrl> {
    let bili_client = app.get_bili_client();
    let media_url = bili_client
        .get_normal_url(&bvid, cid)
        .await
        .map_err(|err| CommandError::from("获取普通视频url失败", err))?;
    Ok(media_url)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_bangumi_url(app: AppHandle, cid: i64) -> CommandResult<BangumiMediaUrl> {
    let bili_client = app.get_bili_client();
    let media_url = bili_client
        .get_bangumi_url(cid)
        .await
        .map_err(|err| CommandError::from("获取番剧视频url失败", err))?;
    Ok(media_url)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_cheese_url(app: AppHandle, ep_id: i64) -> CommandResult<CheeseMediaUrl> {
    let bili_client = app.get_bili_client();
    let media_url = bili_client
        .get_cheese_url(ep_id)
        .await
        .map_err(|err| CommandError::from("获取课程视频url失败", err))?;
    Ok(media_url)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_player_info(app: AppHandle, aid: i64, cid: i64) -> CommandResult<PlayerInfo> {
    let bili_client = app.get_bili_client();
    let player_info = bili_client
        .get_player_info(aid, cid)
        .await
        .map_err(|err| CommandError::from("获取播放器信息失败", err))?;
    Ok(player_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_fav_folders(app: AppHandle, uid: i64) -> CommandResult<FavFolders> {
    let bili_client = app.get_bili_client();
    let fav_folders = bili_client
        .get_fav_folders(uid)
        .await
        .map_err(|err| CommandError::from("获取收藏夹列表失败", err))?;
    Ok(fav_folders)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_fav_info(app: AppHandle, params: GetFavInfoParams) -> CommandResult<FavInfo> {
    let bili_client = app.get_bili_client();
    let fav_info = bili_client
        .get_fav_info(params)
        .await
        .map_err(|err| CommandError::from("获取收藏夹内容失败", err))?;
    Ok(fav_info)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_watch_later_info(app: AppHandle, page: i32) -> CommandResult<WatchLaterInfo> {
    let bili_client = app.get_bili_client();
    let watch_later_info = bili_client
        .get_watch_later_info(page)
        .await
        .map_err(|err| CommandError::from("获取稍后观看内容失败", err))?;
    Ok(watch_later_info)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn create_download_tasks(app: AppHandle, params: CreateDownloadTaskParams) {
    let download_manager = app.get_download_manager();
    download_manager.create_download_tasks(&params);
    tracing::debug!("下载任务创建成功");
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn pause_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.pause_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn resume_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.resume_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn delete_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.delete_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn restart_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.restart_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn restore_download_tasks(app: AppHandle) -> CommandResult<()> {
    let download_manager = app.get_download_manager();
    download_manager
        .restore_download_tasks()
        .map_err(|err| CommandError::from("恢复下载任务失败", err))?;
    tracing::debug!("恢复下载任务成功");
    Ok(())
}
