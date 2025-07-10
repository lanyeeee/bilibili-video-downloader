mod commands;
mod config;
mod errors;
mod events;
mod extensions;
mod logger;
mod types;
mod utils;

use anyhow::Context;
use commands::*;
use config::Config;
use parking_lot::RwLock;
use tauri::{Manager, Wry};

use crate::events::LogEvent;

fn generate_context() -> tauri::Context<Wry> {
    tauri::generate_context!()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<Wry>::new()
        .commands(tauri_specta::collect_commands![greet, get_config])
        .events(tauri_specta::collect_events![LogEvent]);

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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .context("获取app_data_dir目录失败")?;

            std::fs::create_dir_all(&app_data_dir)
                .context(format!("创建app_data_dir目录`{app_data_dir:?}`失败"))?;

            let config = RwLock::new(Config::new(app.handle())?);
            app.manage(config);

            logger::init(app.handle())?;

            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
