use anyhow::Context;
use parking_lot::RwLock;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::{
    config::Config,
    errors::{CommandError, CommandResult},
    extensions::AppHandleExt,
    logger,
    types::{
        bangumi_info::{BangumiInfo, EpInBangumi},
        bangumi_media_url::BangumiMediaUrl,
        cheese_info::CheeseInfo,
        cheese_media_url::CheeseMediaUrl,
        create_download_task_params::CreateDownloadTaskParams,
        fav_folders::FavFolders,
        fav_info::FavInfo,
        get_bangumi_info_params::GetBangumiInfoParams,
        get_cheese_info_params::GetCheeseInfoParams,
        get_fav_info_params::GetFavInfoParams,
        get_normal_info_params::GetNormalInfoParams,
        get_user_video_info_params::GetUserVideoInfoParams,
        normal_info::NormalInfo,
        normal_media_url::NormalMediaUrl,
        player_info::PlayerInfo,
        qrcode_data::QrcodeData,
        qrcode_status::QrcodeStatus,
        search_params::SearchParams,
        search_result::{
            BangumiSearchResult, CheeseSearchResult, FavSearchResult, NormalSearchResult,
            SearchResult, UserVideoSearchResult,
        },
        user_info::UserInfo,
        user_video_info::UserVideoInfo,
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
    let bili_client = app.get_bili_client();
    let config_state = app.get_config();

    let proxy_changed = {
        let config_state = config_state.read();
        config_state.proxy_mode != config.proxy_mode
            || config_state.proxy_host != config.proxy_host
            || config_state.proxy_port != config.proxy_port
    };

    let enable_file_logger = config.enable_file_logger;
    let file_logger_changed = config_state.read().enable_file_logger != enable_file_logger;

    {
        // 包裹在大括号中，以便自动释放写锁
        let mut config_state = config_state.write();
        *config_state = config;
        config_state
            .save(&app)
            .map_err(|err| CommandError::from("保存配置失败", err))?;
        tracing::debug!("保存配置成功");
    }

    if proxy_changed {
        bili_client.reload_client();
    }

    if file_logger_changed {
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

#[tauri::command(async)]
#[specta::specta]
pub async fn search(app: AppHandle, params: SearchParams) -> CommandResult<SearchResult> {
    use SearchParams::{Bangumi, Cheese, Fav, Normal, UserVideo};
    let bili_client = app.get_bili_client();
    let search_result = match params {
        Normal(params) => {
            let info = bili_client
                .get_normal_info(params)
                .await
                .map_err(|err| CommandError::from("获取普通视频信息失败", err))?;
            SearchResult::Normal(NormalSearchResult(info))
        }
        Bangumi(GetBangumiInfoParams::EpId(ep_id)) => {
            let info = bili_client
                .get_bangumi_info(GetBangumiInfoParams::EpId(ep_id))
                .await
                .map_err(|err| CommandError::from("获取番剧视频信息失败", err))?;
            let episodes: Vec<&EpInBangumi> = info
                .episodes
                .iter()
                .chain(
                    info.section
                        .iter()
                        .flat_map(|sections| sections.iter())
                        .flat_map(|section| section.episodes.iter()),
                )
                .collect();
            let ep = episodes.iter().find(|ep| ep.id == ep_id).copied().cloned();
            SearchResult::Bangumi(BangumiSearchResult { ep, info })
        }
        Bangumi(GetBangumiInfoParams::SeasonId(season_id)) => {
            let info = bili_client
                .get_bangumi_info(GetBangumiInfoParams::SeasonId(season_id))
                .await
                .map_err(|err| CommandError::from("获取番剧视频信息失败", err))?;
            SearchResult::Bangumi(BangumiSearchResult { ep: None, info })
        }
        Cheese(GetCheeseInfoParams::EpId(ep_id)) => {
            let info = bili_client
                .get_cheese_info(GetCheeseInfoParams::EpId(ep_id))
                .await
                .map_err(|err| CommandError::from("获取课程视频信息失败", err))?;
            let ep = info.episodes.iter().find(|ep| ep.id == ep_id).cloned();
            SearchResult::Cheese(CheeseSearchResult { ep, info })
        }
        Cheese(GetCheeseInfoParams::SeasonId(season_id)) => {
            let info = bili_client
                .get_cheese_info(GetCheeseInfoParams::SeasonId(season_id))
                .await
                .map_err(|err| CommandError::from("获取课程视频信息失败", err))?;
            SearchResult::Cheese(CheeseSearchResult { ep: None, info })
        }
        UserVideo(params) => {
            let info = bili_client
                .get_user_video_info(params)
                .await
                .map_err(|err| CommandError::from("获取用户视频信息失败", err))?;
            SearchResult::UserVideo(UserVideoSearchResult(info))
        }
        Fav(params) => {
            let info = bili_client
                .get_fav_info(params)
                .await
                .map_err(|err| CommandError::from("获取收藏夹内容失败", err))?;
            SearchResult::Fav(FavSearchResult(info))
        }
    };
    Ok(search_result)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn get_logs_dir_size(app: AppHandle) -> CommandResult<u64> {
    let logs_dir = logger::logs_dir(&app)
        .context("获取日志目录失败")
        .map_err(|err| CommandError::from("获取日志目录大小失败", err))?;
    let logs_dir_size = std::fs::read_dir(&logs_dir)
        .context(format!("读取日志目录`{}`失败", logs_dir.display()))
        .map_err(|err| CommandError::from("获取日志目录大小失败", err))?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum::<u64>();
    tracing::debug!("获取日志目录大小成功");
    Ok(logs_dir_size)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
pub fn show_path_in_file_manager(app: AppHandle, path: &str) -> CommandResult<()> {
    app.opener()
        .reveal_item_in_dir(path)
        .context(format!("在文件管理器中打开`{path}`失败"))
        .map_err(|err| CommandError::from("在文件管理器中打开失败", err))?;
    Ok(())
}
