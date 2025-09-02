use anyhow::Context;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager, State};

use crate::{
    bili_client::BiliClient,
    config::Config,
    downloader::{download_manager::DownloadManager, download_progress::DownloadProgress},
    types::player_info::PlayerInfo,
};

pub trait AnyhowErrorToStringChain {
    /// 将 `anyhow::Error` 转换为chain格式  
    /// # Example  
    /// 0: error message\
    /// 1: error message\
    /// 2: error message
    fn to_string_chain(&self) -> String;
}

impl AnyhowErrorToStringChain for anyhow::Error {
    fn to_string_chain(&self) -> String {
        use std::fmt::Write;
        self.chain()
            .enumerate()
            .fold(String::new(), |mut output, (i, e)| {
                let _ = writeln!(output, "{i}: {e}");
                output
            })
    }
}

pub trait AppHandleExt {
    fn get_config(&self) -> State<RwLock<Config>>;
    fn get_bili_client(&self) -> State<BiliClient>;
    fn get_download_manager(&self) -> State<DownloadManager>;
}

impl AppHandleExt for tauri::AppHandle {
    fn get_config(&self) -> State<RwLock<Config>> {
        self.state::<RwLock<Config>>()
    }
    fn get_bili_client(&self) -> State<BiliClient> {
        self.state::<BiliClient>()
    }
    fn get_download_manager(&self) -> State<DownloadManager> {
        self.state::<DownloadManager>()
    }
}

pub trait GetOrInitPlayerInfo {
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> anyhow::Result<&'a mut PlayerInfo>;
}

impl GetOrInitPlayerInfo for Option<PlayerInfo> {
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> anyhow::Result<&'a mut PlayerInfo> {
        if let Some(info) = self {
            return Ok(info);
        }

        let bili_client = app.get_bili_client();
        let info = bili_client
            .get_player_info(progress.aid, progress.cid)
            .await
            .context("获取播放器信息失败")?;

        Ok(self.insert(info))
    }
}
