use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{config::Config, utils::filename_filter};

use super::episode_type::EpisodeType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmtParams {
    pub task_id: String,
    pub episode_type: EpisodeType,
    pub aid: i64,
    pub bvid: Option<String>,
    pub cid: i64,
    pub ep_id: Option<i64>,
    pub duration: u64,
    pub pub_ts: i64,
    pub collection_title: String,
    pub episode_title: String,
    pub episode_order: i64,
    pub part_title: Option<String>,
    pub part_order: Option<i64>,
    pub up_name: Option<String>,
    pub up_uid: Option<i64>,
    pub create_ts: u64,
}

impl FmtParams {
    pub fn get_episode_dir_and_filename(
        &self,
        config: &Config,
    ) -> anyhow::Result<(PathBuf, String)> {
        use strfmt::strfmt;

        let mut json_value =
            serde_json::to_value(self).context("将FmtParams转为serde_json::Value失败")?;

        let json_map = json_value
            .as_object_mut()
            .context("FmtParams不是JSON对象")?;
        // 格式化时间字段
        format_time_fields(json_map, &config.time_fmt);

        let vars: HashMap<String, String> = json_map
            .into_iter()
            .map(|(k, v)| {
                let key = k.clone();
                let value = match v {
                    Value::String(s) => s.clone(),

                    Value::Null => String::new(),
                    _ => v.to_string(),
                };
                (key, value)
            })
            .collect();

        let dir_fmt = if self.part_title.is_some() {
            &config.dir_fmt_for_part
        } else {
            &config.dir_fmt
        };

        let dir_fmt_parts: Vec<&str> = dir_fmt.split('/').collect();
        let mut dir_names = Vec::new();
        for fmt in dir_fmt_parts {
            let dir_name = strfmt(fmt, &vars).context("格式化目录名失败")?;
            let dir_name = filename_filter(&dir_name);
            if !dir_name.is_empty() {
                dir_names.push(dir_name);
            }
        }

        // 最后一部分是文件名
        let filename = dir_names.pop().context("没有找到文件名部分")?;
        // 剩下的部分是目录名
        let mut episode_dir = config.download_dir.clone();
        for dir_name in dir_names {
            episode_dir = episode_dir.join(dir_name);
        }

        Ok((episode_dir, filename))
    }
}

#[allow(clippy::cast_possible_wrap)]
fn format_time_fields(json_map: &mut Map<String, Value>, time_fmt: &str) {
    if let Some(ts) = json_map.get("pub_ts").and_then(Value::as_i64) {
        if let Some(ts_string) = ts_to_string(ts, time_fmt) {
            json_map.insert("pub_ts".to_string(), Value::String(ts_string));
        }
    }

    if let Some(ts) = json_map.get("create_ts").and_then(Value::as_u64) {
        if let Some(ts_string) = ts_to_string(ts as i64, time_fmt) {
            json_map.insert("create_ts".to_string(), Value::String(ts_string));
        }
    }
}

pub fn ts_to_string(ts: i64, time_fmt: &str) -> Option<String> {
    let ts_string = chrono::DateTime::from_timestamp(ts, 0)?
        .with_timezone(&chrono::Local)
        .format(time_fmt)
        .to_string();
    Some(ts_string)
}
