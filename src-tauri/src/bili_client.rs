use std::time::Duration;

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, Jitter, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{
    http::{HeaderMap, HeaderValue},
    AppHandle,
};

use crate::{
    extensions::AppHandleExt,
    types::{
        bangumi_info::BangumiInfo, bangumi_media_url::BangumiMediaUrl, cheese_info::CheeseInfo,
        get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
        get_normal_info_params::GetNormalInfoParams, normal_info::NormalInfo,
        normal_media_url::NormalMediaUrl, qrcode_data::QrcodeData, qrcode_status::QrcodeStatus,
        user_info::UserInfo,
    },
};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";
const REFERRER: &str = "https://www.bilibili.com/";

pub struct BiliClient {
    app: AppHandle,
    api_client: RwLock<ClientWithMiddleware>,
}

impl BiliClient {
    pub fn new(app: AppHandle) -> Self {
        let api_client = create_api_client(&app);
        let api_client = RwLock::new(api_client);

        Self { app, api_client }
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BiliResp {
    pub code: i64,
    #[serde(default, alias = "message")]
    pub msg: String,
    #[serde(alias = "result")]
    pub data: Option<serde_json::Value>,
}
