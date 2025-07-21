use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::{
    downloader::{download_progress::DownloadProgress, download_task_state::DownloadTaskState},
    types::log_level::LogLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub timestamp: String,
    pub level: LogLevel,
    pub fields: HashMap<String, serde_json::Value>,
    pub target: String,
    pub filename: String,
    #[serde(rename = "line_number")]
    pub line_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    Speed {
        speed: String,
    },

    TaskCreate {
        state: DownloadTaskState,
        progress: DownloadProgress,
    },

    TaskStateUpdate {
        task_id: String,
        state: DownloadTaskState,
    },

    TaskSleeping {
        task_id: String,
        remaining_sec: u64,
    },

    TaskDelete {
        task_id: String,
    },

    ProgressPreparing {
        task_id: String,
    },

    ProgressUpdate {
        progress: DownloadProgress,
    },
}
