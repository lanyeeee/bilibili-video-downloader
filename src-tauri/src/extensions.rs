use parking_lot::RwLock;
use tauri::{Manager, State};

use crate::{bili_client::BiliClient, config::Config};

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
}

impl AppHandleExt for tauri::AppHandle {
    fn get_config(&self) -> State<RwLock<Config>> {
        self.state::<RwLock<Config>>()
    }
    fn get_bili_client(&self) -> State<BiliClient> {
        self.state::<BiliClient>()
    }
}
