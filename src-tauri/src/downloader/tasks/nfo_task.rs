use anyhow::{anyhow, Context};
use chrono::{DateTime, Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use specta::Type;
use yaserde::{YaDeserialize, YaSerialize};

use crate::types::{
    bangumi_info::BangumiInfo, cheese_info::CheeseInfo, normal_info::NormalInfo, tags::Tags,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct NfoTask {
    pub selected: bool,
    pub completed: bool,
}

impl NfoTask {
    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "movie")]
struct Movie {
    title: String,
    plot: String,
    tagline: Option<String>,
    runtime: u64,
    premiered: String,
    year: i32,
    studio: Vec<String>,
    genre: Vec<String>,
    tag: Vec<String>,
    country: Vec<String>,
    set: Option<Set>,
    director: Vec<String>,
    actor: Vec<Actor>,
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "set")]
struct Set {
    name: String,
    overview: String,
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "actor")]
struct Actor {
    name: String,
    role: String,
    order: i64,
    thumb: String,
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "tvshow")]
struct Tvshow {
    title: String,
    plot: String,
    tagline: Option<String>,
    premiered: String,
    year: i32,
    studio: Vec<String>,
    status: String,
    genre: Vec<String>,
    tag: Vec<String>,
    country: Vec<String>,
    director: Vec<String>,
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "episodedetails")]
struct EpisodeDetails {
    title: String,
    plot: String,
    tagline: Option<String>,
    runtime: u64,
    premiered: String,
    year: i32,
    episode: i64,
    studio: Vec<String>,
    genre: Vec<String>,
    tag: Vec<String>,
    country: Vec<String>,
    director: Vec<String>,
}

impl NormalInfo {
    pub fn to_movie_nfo(&self, tags: Tags) -> anyhow::Result<String> {
        let genre = vec![
            "Bilibili视频".to_string(),
            self.tname.clone(),
            self.tname_v2.clone(),
        ];

        let tag: Vec<String> = tags
            .into_iter()
            .map(|t| t.tag_name)
            .filter(|tag_name| !tag_name.is_empty())
            .collect();

        let ts = self.pubdate;
        let date_time = DateTime::from_timestamp(ts, 0)
            .context(format!("将视频发布时间戳转换为日期时间失败: {ts}"))?
            .with_timezone(&chrono::Local);

        let set = self.ugc_season.as_ref().map(|ugc_season| Set {
            name: ugc_season.title.clone(),
            overview: ugc_season.intro.clone(),
        });

        let actor = self.staff.as_ref().map_or(Vec::new(), |staff| {
            staff
                .iter()
                .enumerate()
                .map(|(order, staff)| Actor {
                    name: staff.name.clone(),
                    role: staff.title.clone(),
                    #[allow(clippy::cast_possible_wrap)]
                    order: order as i64,
                    thumb: staff.face.clone(),
                })
                .collect()
        });

        let movie = Movie {
            title: self.title.clone(),
            plot: self.desc.clone(),
            tagline: None,
            runtime: self.duration / 60,
            premiered: date_time.format("%Y-%m-%d").to_string(),
            year: date_time.year(),
            studio: vec!["Bilibili".to_string()],
            genre,
            tag,
            country: Vec::new(),
            set,
            director: vec![self.owner.name.clone()],
            actor,
        };

        let cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };

        let nfo = yaserde::ser::to_string_with_config(&movie, &cfg).map_err(|e| anyhow!(e))?;

        Ok(nfo)
    }
}

impl BangumiInfo {
    pub fn to_tvshow_nfo(&self) -> anyhow::Result<String> {
        let time_str = &self.publish.pub_time;
        let date_time = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").context(
            format!("将番剧发布时间字符串转换为日期时间失败: {time_str}"),
        )?;

        let status = match self.publish.is_finish {
            0 => "Continuing".to_string(),
            _ => "Ended".to_string(),
        };

        let tv_show = Tvshow {
            title: self.title.clone(),
            plot: self.evaluate.clone(),
            tagline: Some(self.share_sub_title.clone()),
            premiered: date_time.format("%Y-%m-%d").to_string(),
            year: date_time.year(),
            studio: vec!["Bilibili".to_string()],
            status,
            genre: self.get_genre(),
            tag: Vec::new(),
            country: self.get_country(),
            director: self.get_director(),
        };

        let cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };

        let nfo = yaserde::ser::to_string_with_config(&tv_show, &cfg).map_err(|e| anyhow!(e))?;

        Ok(nfo)
    }

    pub fn to_episode_details_nfo(&self, ep_id: i64) -> anyhow::Result<String> {
        let (episode, episode_order) = self.get_episode_with_order(ep_id)?;

        let ts = episode.pub_time;
        let date_time = DateTime::from_timestamp(ts, 0)
            .context(format!("将番剧发布时间戳转换为日期时间失败: {ts}"))?
            .with_timezone(&chrono::Local);

        let title = episode
            .show_title
            .clone()
            .context("episode.show_title为None")?;

        let plot = episode
            .share_copy
            .clone()
            .context("episode.share_copy为None")?;

        let duration = episode.duration.context("episode.duration为None")?;

        let episode_details = EpisodeDetails {
            title,
            plot,
            tagline: None,
            runtime: duration / 1000 / 60,
            premiered: date_time.format("%Y-%m-%d").to_string(),
            year: date_time.year(),
            episode: episode_order,
            studio: vec!["Bilibili".to_string()],
            genre: self.get_genre(),
            tag: Vec::new(),
            country: self.get_country(),
            director: self.get_director(),
        };

        let cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };

        let nfo =
            yaserde::ser::to_string_with_config(&episode_details, &cfg).map_err(|e| anyhow!(e))?;

        Ok(nfo)
    }

    fn get_director(&self) -> Vec<String> {
        if let Some(up_info) = &self.up_info {
            vec![up_info.uname.clone()]
        } else {
            Vec::new()
        }
    }

    fn get_country(&self) -> Vec<String> {
        self.areas
            .iter()
            .filter(|area| !area.name.is_empty())
            .map(|area| area.name.clone())
            .collect()
    }

    fn get_genre(&self) -> Vec<String> {
        let type_name = match self.type_field {
            1 => "番剧",
            2 => "电影",
            3 => "纪录片",
            4 => "国创",
            5 => "电视剧",
            6 => "漫画",
            7 => "综艺",
            _ => "",
        };

        let mut genre = Vec::new();
        if !type_name.is_empty() {
            genre.push(format!("Bilibili{type_name}"));
        }

        for style in &self.styles {
            if !style.is_empty() {
                genre.push(style.clone());
            }
        }

        genre
    }
}

impl CheeseInfo {
    pub fn to_tvshow_nfo(&self) -> anyhow::Result<String> {
        let episode = self.episodes.first().context("episodes列表为空")?;
        let ts = episode.release_date;
        let date_time = DateTime::from_timestamp(ts, 0)
            .context(format!("将课程的发布时间戳转换为日期时间失败: {ts}"))?
            .with_timezone(&chrono::Local);

        let status = match self.release_status.as_str() {
            "已完结" => "Ended".to_string(),
            _ => "Continuing".to_string(),
        };

        let tv_show = Tvshow {
            title: self.title.clone(),
            plot: self.subtitle.clone(),
            tagline: None,
            premiered: date_time.format("%Y-%m-%d").to_string(),
            year: date_time.year(),
            studio: vec!["Bilibili".to_string()],
            status,
            genre: vec!["Bilibili课程".to_string()],
            tag: Vec::new(),
            country: Vec::new(),
            director: vec![self.up_info.uname.clone()],
        };

        let cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };

        let nfo = yaserde::ser::to_string_with_config(&tv_show, &cfg).map_err(|e| anyhow!(e))?;

        Ok(nfo)
    }

    pub fn to_episode_details_nfo(&self, ep_id: i64) -> anyhow::Result<String> {
        let episode = self
            .episodes
            .iter()
            .find(|ep| ep.id == ep_id)
            .context(format!("找不到ep_id为`{ep_id}`的课程"))?;

        let ts = episode.release_date;
        let date_time = DateTime::from_timestamp(ts, 0)
            .context(format!("将课程发布时间戳转换为日期时间失败: {ts}"))?
            .with_timezone(&chrono::Local);

        let episode_details = EpisodeDetails {
            title: episode.title.clone(),
            plot: episode.subtitle.clone(),
            tagline: None,
            runtime: episode.duration / 60,
            premiered: date_time.format("%Y-%m-%d").to_string(),
            year: date_time.year(),
            episode: episode.index,
            studio: vec!["Bilibili".to_string()],
            genre: vec!["Bilibili课程".to_string()],
            tag: Vec::new(),
            country: Vec::new(),
            director: vec![self.up_info.uname.clone()],
        };

        let cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };

        let nfo =
            yaserde::ser::to_string_with_config(&episode_details, &cfg).map_err(|e| anyhow!(e))?;

        Ok(nfo)
    }
}
