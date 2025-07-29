use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context};
use md5::{Digest, Md5};
use serde::Deserialize;

use crate::bili_client::{BiliClient, BiliResp};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Deserialize)]
struct WbiImgRespData {
    img_url: String,
    sub_url: String,
}

#[derive(Deserialize)]
struct WeiRespData {
    wbi_img: WbiImgRespData,
}

impl BiliClient {
    // 为请求参数进行 wbi 签名
    pub(crate) async fn wbi(&self, params: &mut Vec<(&str, String)>) -> anyhow::Result<()> {
        let (img_key, sub_key) = self.get_wbi_keys().await.context("获取wbi keys失败")?;
        let mixin_key = get_mixin_key((img_key + &sub_key).as_bytes());

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        // 添加当前时间戳
        params.push(("wts", timestamp.to_string()));
        // 重新排序
        params.sort_by(|a, b| a.0.cmp(b.0));
        // 拼接参数
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", get_url_encoded(k), get_url_encoded(v)))
            .collect::<Vec<_>>()
            .join("&");
        // 计算签名
        let web_sign = format!("{:x}", Md5::digest(query.clone() + &mixin_key));
        params.push(("w_rid", web_sign));
        Ok(())
    }

    async fn get_wbi_keys(&self) -> anyhow::Result<(String, String)> {
        let request = self
            .api_client
            .read()
            .get("https://api.bilibili.com/x/web-interface/nav")
            .header("Cookie", &self.get_cookie());

        let http_resp = request.send().await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!("预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为BiliResp
        let bili_resp: BiliResp =
            serde_json::from_str(&body).context(format!("将body解析为BiliResp失败: {body}"))?;
        // 检查BiliResp的data是否存在
        let Some(data) = bili_resp.data else {
            return Err(anyhow!("BiliResp中不存在data字段: {bili_resp:?}"));
        };
        // 尝试将data解析为Data
        let data_str = data.to_string();
        let wei_resp_data: WeiRespData =
            serde_json::from_str(&data_str).context(format!("将data解析为Data失败: {data_str}"))?;

        let img_url = wei_resp_data.wbi_img.img_url;
        let sub_url = wei_resp_data.wbi_img.sub_url;

        let img_filename =
            take_filename(&img_url).context(format!("从img_url中提取文件名失败: {img_url}"))?;
        let sub_filename =
            take_filename(&sub_url).context(format!("从sub_url中提取文件名失败: {sub_url}"))?;

        Ok((img_filename, sub_filename))
    }
}

// 对 imgKey 和 subKey 进行字符顺序打乱编码
fn get_mixin_key(orig: &[u8]) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| orig[i] as char)
        .collect::<String>()
}

fn get_url_encoded(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                Some(c.to_string())
            } else {
                // 过滤 value 中的 "!'()*" 字符
                if "!'()*".contains(c) {
                    return None;
                }
                let encoded = c
                    .encode_utf8(&mut [0; 4])
                    .bytes()
                    .fold(String::new(), |acc, b| acc + &format!("%{b:02X}"));
                Some(encoded)
            }
        })
        .collect::<String>()
}

fn take_filename(url: &str) -> Option<String> {
    url.rsplit_once('/')
        .and_then(|(_, s)| s.rsplit_once('.'))
        .map(|(s, _)| s.to_string())
}
