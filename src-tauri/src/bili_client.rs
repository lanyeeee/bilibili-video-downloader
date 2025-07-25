use std::time::Duration;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use parking_lot::RwLock;
use prost::Message;
use reqwest::{Client, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, Jitter, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{
    http::{HeaderMap, HeaderValue},
    AppHandle,
};
use tokio::task::JoinSet;

use crate::{
    extensions::AppHandleExt,
    protobuf::DmSegMobileReply,
    types::{
        bangumi_info::BangumiInfo, bangumi_media_url::BangumiMediaUrl, cheese_info::CheeseInfo,
        cheese_media_url::CheeseMediaUrl, fav_folders::FavFolders, fav_info::FavInfo,
        get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
        get_fav_info_params::GetFavInfoParams, get_normal_info_params::GetNormalInfoParams,
        normal_info::NormalInfo, normal_media_url::NormalMediaUrl, player_info::PlayerInfo,
        qrcode_data::QrcodeData, qrcode_status::QrcodeStatus, subtitle::Subtitle,
        user_info::UserInfo, watch_later_info::WatchLaterInfo,
    },
};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";
const REFERRER: &str = "https://www.bilibili.com/";

pub struct BiliClient {
    app: AppHandle,
    api_client: RwLock<ClientWithMiddleware>,
    media_client: RwLock<ClientWithMiddleware>,
    content_length_client: RwLock<Client>,
}

impl BiliClient {
    pub fn new(app: AppHandle) -> Self {
        let api_client = create_api_client(&app);
        let api_client = RwLock::new(api_client);

        let media_client = create_media_client(&app);
        let media_client = RwLock::new(media_client);

        let content_length_client = create_content_length_client(&app);
        let content_length_client = RwLock::new(content_length_client);

        Self {
            app,
            api_client,
            media_client,
            content_length_client,
        }
    }

    pub async fn generate_qrcode(&self) -> anyhow::Result<QrcodeData> {
        // 发送生成二维码请求
        let request = self
            .api_client
            .read()
            .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate");
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为二维码数据
        let data_str = data.to_string();
        let qrcode_data: QrcodeData = serde_json::from_str(&data_str)
            .context(format!("将data解析为QrcodeData失败: {data_str}"))?;

        Ok(qrcode_data)
    }

    pub async fn get_qrcode_status(&self, qrcode_key: &str) -> anyhow::Result<QrcodeStatus> {
        // 发送获取二维码状态请求
        let params = json!({"qrcode_key": qrcode_key});
        let request = self
            .api_client
            .read()
            .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
            .query(&params);
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为二维码状态
        let data_str = data.to_string();
        let qrcode_status: QrcodeStatus = serde_json::from_str(&data_str)
            .context(format!("将data解析为QrcodeStatus失败: {data_str}"))?;
        if ![0, 86101, 86090, 86038].contains(&qrcode_status.code) {
            return Err(anyhow!("预料之外的二维码code: {qrcode_status:?}"));
        }
        Ok(qrcode_status)
    }

    pub async fn get_user_info(&self, sessdata: &str) -> anyhow::Result<UserInfo> {
        // 发送获取用户信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/web-interface/nav")
            .header("cookie", format!("SESSDATA={sessdata}"));
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code == -101 {
            return Err(anyhow!("cookie错误或已过期，请重新登录: {bili_resp:?}"));
        } else if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为UserInfo
        let data_str = data.to_string();
        let user_info: UserInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为UserInfo失败: {data_str}"))?;

        Ok(user_info)
    }

    pub async fn get_normal_info(&self, params: GetNormalInfoParams) -> anyhow::Result<NormalInfo> {
        use GetNormalInfoParams::{Aid, Bvid};
        let params = match params {
            Bvid(bvid) => json!({"bvid": bvid}),
            Aid(aid) => json!({"aid": aid}),
        };
        // 发送获取普通视频信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/web-interface/view")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为NormalInfo
        let data_str = data.to_string();
        let normal_info: NormalInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为NormalInfo失败: {data_str}"))?;

        Ok(normal_info)
    }

    pub async fn get_bangumi_info(
        &self,
        params: GetBangumiInfoParams,
    ) -> anyhow::Result<BangumiInfo> {
        use GetBangumiInfoParams::{EpId, SeasonId};
        let params = match params {
            EpId(ep_id) => json!({"ep_id": ep_id}),
            SeasonId(season_id) => json!({"season_id": season_id}),
        };
        // 发送获取番剧视频信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/pgc/view/web/season")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为BangumiInfo
        let data_str = data.to_string();
        let bangumi_info: BangumiInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为BangumiInfo失败: {data_str}"))?;

        Ok(bangumi_info)
    }

    pub async fn get_cheese_info(&self, params: GetCheeseInfoParams) -> anyhow::Result<CheeseInfo> {
        use GetCheeseInfoParams::{EpId, SeasonId};
        let params = match params {
            EpId(ep_id) => json!({"ep_id": ep_id}),
            SeasonId(season_id) => json!({"season_id": season_id}),
        };
        // 发送获取课程视频信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/pugv/view/web/season")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为CheeseInfo
        let data_str = data.to_string();
        let cheese_info: CheeseInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为CheeseInfo失败: {data_str}"))?;

        Ok(cheese_info)
    }

    pub async fn get_normal_url(&self, bvid: &str, cid: i64) -> anyhow::Result<NormalMediaUrl> {
        let params = json!({
            "bvid": bvid,
            "cid": cid,
            "qn": 127,
            "fnval": 4048,
        });
        // 发送获取普通url的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/player/wbi/playurl")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为NormalMediaUrl
        let data_str = data.to_string();
        let media_url: NormalMediaUrl = serde_json::from_str(&data_str)
            .context(format!("将data解析为NormalMediaUrl失败: {data_str}"))?;

        Ok(media_url)
    }

    pub async fn get_bangumi_url(&self, cid: i64) -> anyhow::Result<BangumiMediaUrl> {
        let params = json!({
            "cid": cid,
            "qn": 127,
            "fnval": 4048,
        });
        // 发送获取番剧url的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/pgc/player/web/playurl")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code == -10403 {
            return Err(anyhow!(
                "地区限制，请使用代理或切换线路后重试: {bili_resp:?}"
            ));
        } else if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为BangumiMediaUrl
        let data_str = data.to_string();
        let media_url: BangumiMediaUrl = serde_json::from_str(&data_str)
            .context(format!("将data解析为BangumiMediaUrl失败: {data_str}"))?;

        Ok(media_url)
    }

    pub async fn get_cheese_url(&self, ep_id: i64) -> anyhow::Result<CheeseMediaUrl> {
        let params = json!({
            "ep_id": ep_id,
            "qn": 127,
            "fnval": 4048,
        });
        // 发送获取课程url的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/pugv/player/web/playurl")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code == -403 {
            return Err(anyhow!("没有观看权限，请先购买: {bili_resp:?}"));
        } else if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为CheeseMediaUrl
        let data_str = data.to_string();
        let media_url: CheeseMediaUrl = serde_json::from_str(&data_str)
            .context(format!("将data解析为CheeseMediaUrl失败: {data_str}"))?;

        Ok(media_url)
    }

    pub async fn get_player_info(&self, aid: i64, cid: i64) -> anyhow::Result<PlayerInfo> {
        let params = json!({
            "aid": aid,
            "cid": cid,
        });
        // 发送获取播放器信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/player/wbi/v2")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为PlayerInfo
        let data_str = data.to_string();
        let player_info: PlayerInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为PlayerInfo失败: {data_str}"))?;

        Ok(player_info)
    }

    pub async fn get_fav_folders(&self, uid: i64) -> anyhow::Result<FavFolders> {
        let params = json!({"up_mid": uid});
        // 发送获取收藏夹信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/v3/fav/folder/created/list-all")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为FavFolders
        let data_str = data.to_string();
        let fav_folders: FavFolders = serde_json::from_str(&data_str)
            .context(format!("将data解析为FavFolders失败: {data_str}"))?;

        Ok(fav_folders)
    }

    pub async fn get_fav_info(&self, params: GetFavInfoParams) -> anyhow::Result<FavInfo> {
        let params = json!({
            "media_id": params.media_list_id,
            "pn": params.pn,
            "ps": 36,
            "platform": "web",
        });
        // 发送获取收藏夹信息的请求
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/v3/fav/resource/list")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为FavInfo
        let data_str = data.to_string();
        let fav_info: FavInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为FavInfo失败: {data_str}"))?;

        Ok(fav_info)
    }

    pub async fn get_watch_later_info(&self, page: i32) -> anyhow::Result<WatchLaterInfo> {
        // 发送获取稍后观看信息的请求
        let params = json!({"ps": 20, "pn": page});
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/v2/history/toview")
            .query(&params)
            .header("cookie", self.get_cookie());
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的code字段
        if bili_resp.code != 0 {
            return Err(anyhow!("预料之外的code: {bili_resp:?}"));
        }
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为WatchLaterInfo
        let data_str = data.to_string();
        let watch_later_info: WatchLaterInfo = serde_json::from_str(&data_str)
            .context(format!("将data解析为WatchLaterInfo失败: {data_str}"))?;

        Ok(watch_later_info)
    }

    pub async fn get_media_chunk(
        &self,
        media_url: &str,
        start: u64,
        end: u64,
    ) -> anyhow::Result<Bytes> {
        let request = self
            .media_client
            .read()
            .get(media_url)
            .header("range", format!("bytes={start}-{end}"));
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        if status != StatusCode::PARTIAL_CONTENT {
            return Err(anyhow!("预料之外的状态码({status})"));
        }

        let bytes = http_resp.bytes().await?;

        Ok(bytes)
    }

    pub async fn get_content_length(&self, media_url: &str) -> anyhow::Result<u64> {
        let request = self.content_length_client.read().head(media_url);
        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status})"));
        }

        let headers = http_resp.headers();
        let content_length = headers
            .get("Content-Length")
            .context("缺少 Content-Length 响应头")?
            .to_str()
            .context("Content-Length 响应头无法转换为字符串")?
            .parse::<u64>()
            .context("Content-Length 响应头无法转换为整数")?;

        Ok(content_length)
    }

    pub async fn get_url_with_content_length(&self, urls: Vec<String>) -> Vec<(String, u64)> {
        let mut url_with_content_length = Vec::new();
        let mut join_set = JoinSet::new();

        for url in urls {
            let app = self.app.clone();
            join_set.spawn(async move {
                let bili_client = app.get_bili_client();
                let Ok(content_length) = bili_client.get_content_length(&url).await else {
                    return None;
                };
                Some((url, content_length))
            });
        }

        while let Some(Ok(Some((url, content_length)))) = join_set.join_next().await {
            url_with_content_length.push((url, content_length));
        }

        url_with_content_length
    }

    pub async fn get_danmaku(
        &self,
        aid: i64,
        cid: i64,
        duration: u64,
    ) -> anyhow::Result<Vec<DmSegMobileReply>> {
        let client = self.api_client.read().clone();
        // 以6分钟为单位分段
        let segment_count = duration.div_ceil(360);

        let mut join_set = JoinSet::new();
        for segment_index in 1..=segment_count {
            let client = client.clone();
            let cookie = self.get_cookie();

            join_set.spawn(async move {
                // 发送获取分段弹幕的请求
                let params = json!({
                    "type": 1,
                    "oid": cid,
                    "pid": aid,
                    "segment_index": segment_index,
                });
                let http_resp = client
                    .get("https://api.bilibili.com/x/v2/dm/web/seg.so")
                    .query(&params)
                    .header("cookie", cookie)
                    .send()
                    .await?;
                let status = http_resp.status();
                if status != StatusCode::OK {
                    let body = http_resp.text().await?;
                    return Err(anyhow!("预料之外的状态码({status}): {body}"));
                }
                let body = http_resp.bytes().await?;
                let reply =
                    DmSegMobileReply::decode(body).context("将body解析为DmSegMobileReply失败")?;

                Ok(reply)
            });
        }

        let mut replies = Vec::new();
        while let Some(Ok(res)) = join_set.join_next().await {
            let reply = res?;
            replies.push(reply);
        }

        Ok(replies)
    }

    pub async fn get_subtitle(&self, url: &str) -> anyhow::Result<Subtitle> {
        let request = self.api_client.read().get(url);
        let http_resp = request.send().await?;
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为Subtitle
        let subtitle: Subtitle =
            serde_json::from_str(&body).context(format!("将body解析为Subtitle失败: {body}"))?;

        Ok(subtitle)
    }

    fn get_cookie(&self) -> String {
        let sessdata = self.app.get_config().read().sessdata.clone();
        format!("SESSDATA={sessdata}")
    }
}

fn create_api_client(_app: &AppHandle) -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder()
        .base(1)
        .jitter(Jitter::Bounded)
        .build_with_total_retry_duration(Duration::from_secs(5));

    let mut headers = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
    headers.insert("referer", HeaderValue::from_static(REFERRER));

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(3))
        .default_headers(headers)
        .build()
        .unwrap();

    reqwest_middleware::ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

fn create_media_client(_app: &AppHandle) -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder()
        .base(1)
        .jitter(Jitter::Bounded)
        .build_with_max_retries(3);

    let mut headers = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
    headers.insert("referer", HeaderValue::from_static(REFERRER));

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();

    reqwest_middleware::ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

fn create_content_length_client(_app: &AppHandle) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
    headers.insert("referer", HeaderValue::from_static(REFERRER));

    reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .build()
        .unwrap()
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BiliResp {
    pub code: i64,
    #[serde(default, alias = "message")]
    pub msg: String,
    #[serde(alias = "result")]
    pub data: Option<serde_json::Value>,
}
