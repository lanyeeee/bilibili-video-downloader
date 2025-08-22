mod bili_client;
mod commands;
mod config;
mod danmaku_xml_to_ass;
mod downloader;
mod errors;
mod events;
mod extensions;
mod logger;
mod types;
mod utils;
mod wbi;
mod protobuf {
    include!("./bilibili.community.service.dm.v1.rs");
}

use anyhow::Context;
use commands::*;
use config::Config;
use parking_lot::RwLock;
use tauri::{Manager, Wry};

use crate::{
    bili_client::BiliClient,
    downloader::download_manager::DownloadManager,
    events::{DownloadEvent, LogEvent},
};

fn generate_context() -> tauri::Context<Wry> {
    tauri::generate_context!()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<Wry>::new()
        .commands(tauri_specta::collect_commands![
            get_config,
            save_config,
            generate_qrcode,
            get_qrcode_status,
            get_user_info,
            get_normal_info,
            get_bangumi_info,
            get_user_video_info,
            get_fav_folders,
            get_fav_info,
            get_watch_later_info,
            get_bangumi_follow_info,
            create_download_tasks,
            pause_download_tasks,
            resume_download_tasks,
            delete_download_tasks,
            restart_download_tasks,
            restore_download_tasks,
            search,
            get_logs_dir_size,
            show_path_in_file_manager,
        ])
        .events(tauri_specta::collect_events![LogEvent, DownloadEvent]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .formatter(specta_typescript::formatter::prettier)
                .header("// @ts-nocheck"), // disable typescript checks
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    // 解决Ubuntu24.04窗口全白的问题
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .context("获取app_data_dir目录失败")?;

            std::fs::create_dir_all(&app_data_dir).context(format!(
                "创建app_data_dir目录`{:?}`失败",
                app_data_dir.display()
            ))?;

            let config = RwLock::new(Config::new(app.handle())?);
            app.manage(config);

            let bili_client = BiliClient::new(app.handle().clone());
            app.manage(bili_client);

            let download_manager = DownloadManager::new(app.handle().clone());
            app.manage(download_manager);

            logger::init(app.handle())?;

            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
