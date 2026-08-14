use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use eyre::WrapErr;
use parking_lot::RwLock;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tracing::instrument;

use crate::{
    config::Config,
    downloader::download_task::RestoredDownloadTask,
    errors::{CommandError, CommandResult},
    extensions::AppHandleExt,
    logger,
    types::{
        available_media_formats::AvailableMediaFormats,
        bangumi_follow_info::BangumiFollowInfo,
        bangumi_info::{BangumiInfo, EpInBangumi},
        create_download_task_params::CreateDownloadTaskParams,
        fav_folders::FavFolders,
        fav_info::FavInfo,
        get_available_media_formats_params::GetAvailableMediaFormatsParams,
        get_bangumi_follow_info_params::GetBangumiFollowInfoParams,
        get_bangumi_info_params::GetBangumiInfoParams,
        get_cheese_info_params::GetCheeseInfoParams,
        get_fav_info_params::GetFavInfoParams,
        get_history_info_params::GetHistoryInfoParams,
        get_normal_info_params::GetNormalInfoParams,
        get_user_video_info_params::GetUserVideoInfoParams,
        history_info::HistoryInfo,
        log_metadata::LogMetadata,
        normal_info::NormalInfo,
        plugin_info::PluginInfo,
        qrcode_data::QrcodeData,
        qrcode_status::QrcodeStatus,
        restart_download_task_params::RestartDownloadTaskParams,
        search_params::SearchParams,
        search_result::{
            BangumiSearchResult, CheeseSearchResult, FavSearchResult, NormalSearchResult,
            SearchResult, UserVideoSearchResult,
        },
        skip_segments::SkipSegments,
        user_info::UserInfo,
        user_video_info::UserVideoInfo,
        watch_later_info::WatchLaterInfo,
    },
};

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
#[instrument(level = "error", skip_all)]
// TODO: 改成 app.get_config_manager().read().clone()
pub fn get_config(config: tauri::State<RwLock<Config>>) -> Config {
    config.read().clone()
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
#[instrument(level = "error", skip_all)]
pub fn save_config(app: AppHandle, config: Config) -> CommandResult<()> {
    let bili_client = app.get_bili_client();
    let config_state = app.get_config();

    let enable_file_logger = config.enable_file_logger;
    let (proxy_changed, file_logger_changed) = {
        let current_config = config_state.read();
        (
            current_config.proxy_mode != config.proxy_mode
                || current_config.proxy_host != config.proxy_host
                || current_config.proxy_port != config.proxy_port,
            current_config.enable_file_logger != enable_file_logger,
        )
    };

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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all, fields(ep_id = params.get_ep_id(), season_id = params.get_season_id()))]
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
#[instrument(level = "error", skip_all, fields(bvid = params.get_bvid(), aid = params.get_aid()))]
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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
pub async fn get_watch_later_info(app: AppHandle, page: i32) -> CommandResult<WatchLaterInfo> {
    let bili_client = app.get_bili_client();
    let watch_later_info = bili_client
        .get_watch_later_info(page)
        .await
        .map_err(|err| CommandError::from("获取稍后观看内容失败", err))?;
    Ok(watch_later_info)
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub async fn get_bangumi_follow_info(
    app: AppHandle,
    params: GetBangumiFollowInfoParams,
) -> CommandResult<BangumiFollowInfo> {
    let bili_client = app.get_bili_client();
    let bangumi_follow_info = bili_client
        .get_bangumi_follow_info(params)
        .await
        .map_err(|err| CommandError::from("获取追番信息失败", err))?;
    Ok(bangumi_follow_info)
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub async fn get_history_info(
    app: AppHandle,
    params: GetHistoryInfoParams,
) -> CommandResult<HistoryInfo> {
    let bili_client = app.get_bili_client();
    let history_info = bili_client
        .get_history_info(params)
        .await
        .map_err(|err| CommandError::from("获取历史记录失败", err))?;
    Ok(history_info)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn create_download_tasks(app: AppHandle, params: CreateDownloadTaskParams) {
    let download_manager = app.get_download_manager();
    download_manager.create_download_tasks(&params);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn pause_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.pause_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn resume_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.resume_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn delete_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.delete_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn restart_download_tasks(app: AppHandle, task_ids: Vec<String>) {
    let download_manager = app.get_download_manager();
    download_manager.restart_download_tasks(&task_ids);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(task_id = params.task_id))]
pub fn restart_download_task(app: AppHandle, params: RestartDownloadTaskParams) {
    let download_manager = app.get_download_manager();
    download_manager.restart_download_task(&params);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub async fn restore_download_tasks(app: AppHandle) -> CommandResult<Vec<RestoredDownloadTask>> {
    let download_manager = app.get_download_manager();
    let restored_tasks = download_manager
        .restore_download_tasks()
        .await
        .map_err(|err| CommandError::from("恢复下载任务失败", err))?;
    tracing::debug!("恢复下载任务成功");
    Ok(restored_tasks)
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
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
#[instrument(level = "error", skip_all)]
pub fn get_logs_dir_size(app: AppHandle) -> CommandResult<u64> {
    let logs_dir = logger::logs_dir(&app)
        .wrap_err("获取日志目录失败")
        .map_err(|err| CommandError::from("获取日志目录大小失败", err))?;
    let logs_dir_size = std::fs::read_dir(&logs_dir)
        .wrap_err(format!("读取日志目录`{}`失败", logs_dir.display()))
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
#[instrument(level = "error", skip_all)]
pub fn show_path_in_file_manager(app: AppHandle, path: &str) -> CommandResult<()> {
    app.opener()
        .reveal_item_in_dir(path)
        .wrap_err(format!("在文件管理器中打开`{path}`失败"))
        .map_err(|err| CommandError::from("在文件管理器中打开失败", err))?;
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(bvid = bvid, cid = cid))]
pub async fn get_skip_segments(
    app: AppHandle,
    bvid: String,
    cid: Option<i64>,
) -> CommandResult<SkipSegments> {
    let bili_client = app.get_bili_client();
    let skip_segments = bili_client
        .get_skip_segments(&bvid, cid)
        .await
        .map_err(|err| CommandError::from("获取跳过片段失败", err))?;
    Ok(skip_segments)
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub async fn get_available_media_formats(
    app: AppHandle,
    params: GetAvailableMediaFormatsParams,
) -> CommandResult<AvailableMediaFormats> {
    let bili_client = app.get_bili_client();
    let result = match params {
        GetAvailableMediaFormatsParams::Normal(params) => {
            let media_url = bili_client
                .get_normal_url(&params.bvid, params.cid)
                .await
                .map_err(|err| CommandError::from("获取普通视频可用格式失败", err))?;

            media_url.to_get_available_media_formats_result()
        }
        GetAvailableMediaFormatsParams::Bangumi(params) => {
            let media_url = bili_client
                .get_bangumi_url(params.cid)
                .await
                .map_err(|err| CommandError::from("获取番剧视频可用格式失败", err))?;

            media_url.to_get_available_media_formats_result()
        }
        GetAvailableMediaFormatsParams::Cheese(params) => {
            let media_url = bili_client
                .get_cheese_url(params.ep_id)
                .await
                .map_err(|err| CommandError::from("获取课程视频可用格式失败", err))?;

            media_url.to_get_available_media_formats_result()
        }
    };

    Ok(result)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(path = path))]
pub fn open_log_file(path: &str) -> CommandResult<Vec<LogMetadata>> {
    let log_file = File::open(path).map_err(|err| CommandError::from("打开日志文件失败", err))?;

    let reader = BufReader::new(log_file);

    let mut logs = Vec::new();
    let mut line_num = 0;

    for line_result in reader.lines() {
        line_num += 1;

        let line = line_result
            .wrap_err(format!("读取日志文件的第`{line_num}`行失败"))
            .map_err(|err| CommandError::from("打开日志文件失败", err))?;

        if line.trim().is_empty() {
            continue;
        }

        let log: LogMetadata = serde_json::from_str(&line)
            .wrap_err(format!("将日志文件的第`{line_num}`行解析为LogMetadata失败"))
            .map_err(|err| CommandError::from("打开日志文件失败", err))?;

        logs.push(log);
    }

    Ok(logs)
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all)]
pub fn get_plugin_infos(app: AppHandle) -> Vec<PluginInfo> {
    app.get_plugin_manager().get_plugin_infos()
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(plugin_path = plugin_path))]
pub fn add_plugin(app: AppHandle, plugin_path: String) -> CommandResult<()> {
    let plugin_manager = app.get_plugin_manager();

    plugin_manager
        .add_plugin(&plugin_path)
        .map_err(|err| CommandError::from("加载插件失败", err))?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(plugin_path = plugin_path))]
pub fn uninstall_plugin(app: AppHandle, plugin_path: String) -> CommandResult<()> {
    let plugin_manager = app.get_plugin_manager();

    plugin_manager
        .uninstall_plugin(&plugin_path)
        .map_err(|err| CommandError::from("卸载插件失败", err))?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(plugin_path = plugin_path, enabled = enabled))]
pub fn set_plugin_enabled(app: AppHandle, plugin_path: String, enabled: bool) -> CommandResult<()> {
    let plugin_manager = app.get_plugin_manager();

    plugin_manager
        .set_plugin_enabled(&plugin_path, enabled)
        .map_err(|err| CommandError::from("设置插件启用状态失败", err))?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
#[specta::specta]
#[instrument(level = "error", skip_all, fields(plugin_path = plugin_path, priority = priority))]
pub fn set_plugin_priority(
    app: AppHandle,
    plugin_path: String,
    priority: i32,
) -> CommandResult<()> {
    let plugin_manager = app.get_plugin_manager();

    plugin_manager
        .set_plugin_priority(&plugin_path, priority)
        .map_err(|err| CommandError::from("设置插件优先级失败", err))?;

    Ok(())
}
