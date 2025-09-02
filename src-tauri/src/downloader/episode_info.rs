use anyhow::Context;
use tauri::AppHandle;

use crate::{
    downloader::{download_progress::DownloadProgress, episode_type::EpisodeType},
    extensions::AppHandleExt,
    types::{
        bangumi_info::BangumiInfo, cheese_info::CheeseInfo,
        get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
        get_normal_info_params::GetNormalInfoParams, normal_info::NormalInfo,
    },
};

#[derive(Clone)]
pub enum EpisodeInfo {
    Normal(NormalInfo),
    Bangumi(BangumiInfo, i64),
    Cheese(CheeseInfo, i64),
}

pub trait GetOrInitEpisodeInfo {
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> anyhow::Result<&'a mut EpisodeInfo>;
}

impl GetOrInitEpisodeInfo for Option<EpisodeInfo> {
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> anyhow::Result<&'a mut EpisodeInfo> {
        if let Some(info) = self {
            return Ok(info);
        }

        let bili_client = app.get_bili_client();
        let (aid, ep_id, episode_type) = (progress.aid, progress.ep_id, progress.episode_type);

        let new_info = match episode_type {
            EpisodeType::Normal => {
                let info = bili_client
                    .get_normal_info(GetNormalInfoParams::Aid(aid))
                    .await
                    .context("获取普通视频信息失败")?;
                EpisodeInfo::Normal(info)
            }
            EpisodeType::Bangumi => {
                let ep_id = ep_id.context("ep_id为None")?;
                let info = bili_client
                    .get_bangumi_info(GetBangumiInfoParams::EpId(ep_id))
                    .await
                    .context("获取番剧信息失败")?;
                EpisodeInfo::Bangumi(info, ep_id)
            }
            EpisodeType::Cheese => {
                let ep_id = ep_id.context("ep_id为None")?;
                let info = bili_client
                    .get_cheese_info(GetCheeseInfoParams::EpId(ep_id))
                    .await
                    .context("获取课程信息失败")?;
                EpisodeInfo::Cheese(info, ep_id)
            }
        };

        Ok(self.insert(new_info))
    }
}
