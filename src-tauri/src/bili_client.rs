use std::time::Duration;

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, Jitter, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use tauri::{
    http::{HeaderMap, HeaderValue},
    AppHandle,
};

use crate::types::qrcode_data::QrcodeData;

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
