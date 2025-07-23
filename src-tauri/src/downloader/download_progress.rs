use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    config::Config,
    downloader::tasks::{audio_task::AudioTask, merge_task::MergeTask, video_task::VideoTask},
    extensions::AppHandleExt,
    types::{
        audio_quality::AudioQuality,
        bangumi_info::BangumiInfo,
        cheese_info::CheeseInfo,
        codec_type::CodecType,
        normal_info::{NormalInfo, UgcSeason},
        video_quality::VideoQuality,
    },
};

use super::{episode_type::EpisodeType, fmt_params::FmtParams};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct DownloadProgress {
    pub task_id: String,
    pub episode_type: EpisodeType,
    pub aid: i64,
    pub bvid: Option<String>,
    pub cid: i64,
    pub ep_id: Option<i64>,
    pub duration: u64,
    pub pub_ts: i64,
    pub collection_title: String,
    pub part_title: Option<String>,
    pub part_order: Option<i64>,
    pub episode_title: String,
    pub episode_order: i64,
    pub up_name: Option<String>,
    pub up_uid: Option<i64>,
    pub up_avatar: Option<String>,
    pub episode_dir: PathBuf,
    pub filename: String,
    pub video_task: VideoTask,
    pub audio_task: AudioTask,
    pub merge_task: MergeTask,
    pub create_ts: u64,
    pub completed_ts: Option<u64>,
}

impl DownloadProgress {
    pub fn from_normal(
        app: &AppHandle,
        info: &NormalInfo,
        aid: i64,
        cid: Option<i64>,
    ) -> anyhow::Result<Vec<Self>> {
        let config = app.get_config().read().clone();

        if let Some(ugc_season) = &info.ugc_season {
            create_normal_progresses_for_season(ugc_season, info, aid, cid, &config)
        } else {
            create_normal_progresses_for_single(info, cid, &config)
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn from_bangumi(app: &AppHandle, info: &BangumiInfo, ep_id: i64) -> anyhow::Result<Self> {
        let (episode, episode_order) = info.get_episode_with_order(ep_id)?;
        let Some(duration) = episode.duration else {
            return Err(anyhow!("找不到ep_id为`{ep_id}`的番剧的时长"));
        };
        // 将毫秒转换为秒
        let duration = duration / 1000;

        let config = app.get_config().read().clone();

        let tasks = Tasks::new(&config, &episode.cover);

        let (up_name, up_uid, up_avatar) = if let Some(up_info) = &info.up_info {
            (
                Some(up_info.uname.clone()),
                Some(up_info.mid),
                Some(up_info.avatar.clone()),
            )
        } else {
            (None, None, None)
        };

        let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut progress = Self {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Bangumi,
            aid: episode.aid,
            bvid: episode.bvid.clone(),
            cid: episode.cid,
            ep_id: Some(episode.id),
            duration,
            pub_ts: episode.pub_time,
            collection_title: info.title.clone(),
            part_title: None,
            part_order: None,
            episode_title: episode.show_title.clone().unwrap_or(episode.title.clone()),
            episode_order,
            up_name,
            up_uid,
            up_avatar,
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(&config)
            .context("更新需要格式化的字段失败")?;

        Ok(progress)
    }

    pub fn from_cheese(app: &AppHandle, info: &CheeseInfo, ep_id: i64) -> anyhow::Result<Self> {
        let episode = info
            .episodes
            .iter()
            .find(|ep| ep.id == ep_id)
            .context(format!("找不到ep_id为`{ep_id}`的课程"))?;

        let config = app.get_config().read().clone();

        let tasks = Tasks::new(&config, &episode.cover);

        let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut progress = Self {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Cheese,
            aid: episode.aid,
            bvid: None,
            cid: episode.cid,
            ep_id: Some(episode.id),
            duration: episode.duration,
            pub_ts: episode.release_date,
            collection_title: info.title.clone(),
            part_title: None,
            part_order: None,
            episode_title: episode.title.clone(),
            episode_order: episode.index,
            up_name: Some(info.up_info.uname.clone()),
            up_uid: Some(info.up_info.mid),
            up_avatar: Some(info.up_info.avatar.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(&config)
            .context("更新需要格式化的字段失败")?;

        Ok(progress)
    }

    pub async fn prepare(&mut self, app: &AppHandle) -> anyhow::Result<()> {
        let video_selected = self.video_task.selected;
        let video_completed = self.video_task.completed;
        let audio_selected = self.audio_task.selected;
        let audio_completed = self.audio_task.completed;

        if (!video_selected && !audio_selected) || (video_completed && audio_completed) {
            // 如果视频和音频都没有选中，或者都已经完成，则不需要准备
            return Ok(());
        }

        let bili_client = app.get_bili_client();

        match self.episode_type {
            EpisodeType::Normal => {
                let Some(bvid) = &self.bvid else {
                    return Err(anyhow!("progress中的bvid为None，无法获取视频链接"));
                };
                let media_url = bili_client
                    .get_normal_url(bvid, self.cid)
                    .await
                    .context("获取视频链接失败")?;

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_normal(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_normal(app, &media_url).await?;
                }
            }
            EpisodeType::Bangumi => {
                let media_url = bili_client
                    .get_bangumi_url(self.cid)
                    .await
                    .context("获取番剧视频链接失败")?;

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_bangumi(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_bangumi(app, &media_url).await?;
                }
            }
            EpisodeType::Cheese => {
                let Some(ep_id) = self.ep_id else {
                    return Err(anyhow!("progress中的ep_id为None，无法获取课程视频链接"));
                };
                let media_url = bili_client
                    .get_cheese_url(ep_id)
                    .await
                    .context("获取课程视频链接失败")?;

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_cheese(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_cheese(app, &media_url).await?;
                }
            }
        }

        Ok(())
    }

    fn update_fmt_fields(&mut self, config: &Config) -> anyhow::Result<()> {
        let fmt_params = self.create_fmt_params();

        let (episode_dir, filename) = fmt_params.get_episode_dir_and_filename(config)?;

        self.episode_dir = episode_dir;
        self.filename = filename;

        Ok(())
    }

    fn create_fmt_params(&self) -> FmtParams {
        FmtParams {
            task_id: self.task_id.clone(),
            episode_type: self.episode_type,
            aid: self.aid,
            bvid: self.bvid.clone(),
            cid: self.cid,
            ep_id: self.ep_id,
            duration: self.duration,
            pub_ts: self.pub_ts,
            collection_title: self.collection_title.clone(),
            episode_title: self.episode_title.clone(),
            episode_order: self.episode_order,
            part_title: self.part_title.clone(),
            part_order: self.part_order,
            up_name: self.up_name.clone(),
            up_uid: self.up_uid,
            create_ts: self.create_ts,
        }
    }

    pub fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let progress = self.clone();
        let file_name = format!("{}.json", progress.task_id);

        let app_data_dir = app.path().app_data_dir()?;
        let task_dir = app_data_dir.join(".下载任务");
        std::fs::create_dir_all(&task_dir)?;

        let save_path = task_dir.join(file_name);
        let progress_json = serde_json::to_string(&progress)?;
        std::fs::write(save_path, progress_json)?;

        Ok(())
    }

    pub fn is_completed(&self) -> bool {
        self.video_task.is_completed()
            && self.audio_task.is_completed()
            && self.merge_task.is_completed()
    }

    pub fn mark_uncompleted(&mut self) {
        self.video_task.mark_uncompleted();
        self.audio_task.mark_uncompleted();
        self.merge_task.completed = false;
    }

    pub fn get_ids_string(&self) -> String {
        let aid = self.aid;
        let bvid = self.bvid.as_deref().unwrap_or("None");
        let cid = self.cid;
        let ep_id = self.ep_id.map_or("None".to_string(), |id| id.to_string());
        format!("aid: {aid}, bvid: {bvid}, cid: {cid}, ep_id: {ep_id}")
    }
}

#[allow(clippy::too_many_lines)]
fn create_normal_progresses_for_single(
    info: &NormalInfo,
    cid: Option<i64>,
    config: &Config,
) -> anyhow::Result<Vec<DownloadProgress>> {
    let tasks = Tasks::new(config, &info.pic);

    let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if let Some(cid) = cid {
        // 如果有cid，则说明是要下载单个分P
        let Some(page) = info.pages.iter().find(|p| p.cid == cid) else {
            return Err(anyhow!("找不到cid为`{cid}`的分P"));
        };
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: info.aid,
            bvid: Some(info.bvid.clone()),
            cid: page.cid,
            ep_id: None,
            duration: page.duration,
            pub_ts: info.pubdate,
            collection_title: info.title.clone(),
            part_title: Some(page.part.clone()),
            part_order: Some(page.page),
            episode_title: info.title.clone(),
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        return Ok(vec![progress]);
    }

    if info.pages.len() == 1 {
        // 如果只有一个分P，则直接创建一个progress
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: info.aid,
            bvid: Some(info.bvid.clone()),
            cid: info.cid,
            ep_id: None,
            duration: info.duration,
            pub_ts: info.pubdate,
            collection_title: info.title.clone(),
            part_title: None,
            part_order: None,
            episode_title: info.title.clone(),
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        return Ok(vec![progress]);
    }
    // 如果有多个分P，则为每个分P创建一个progress
    let mut progresses = Vec::new();
    for page in &info.pages {
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: info.aid,
            bvid: Some(info.bvid.clone()),
            cid: page.cid,
            ep_id: None,
            duration: page.duration,
            pub_ts: info.pubdate,
            collection_title: info.title.clone(),
            part_title: Some(page.part.clone()),
            part_order: Some(page.page),
            episode_title: info.title.clone(),
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video.clone(),
            audio_task: tasks.audio.clone(),
            merge_task: tasks.merge.clone(),
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        progresses.push(progress);
    }
    Ok(progresses)
}

#[allow(clippy::too_many_lines)]
fn create_normal_progresses_for_season(
    ugc_season: &UgcSeason,
    info: &NormalInfo,
    aid: i64,
    cid: Option<i64>,
    config: &Config,
) -> anyhow::Result<Vec<DownloadProgress>> {
    let section_index = ugc_season
        .sections
        .iter()
        .position(|s| s.episodes.iter().any(|e| e.aid == aid))
        .context(format!("找不到含有aid为`{aid}`的ep的section"))?;
    let section = &ugc_season.sections[section_index];
    #[allow(clippy::cast_possible_wrap)]
    let (ep, episode_order) = section
        .episodes
        .iter()
        .enumerate()
        .map(|(i, e)| (e, i as i64 + 1))
        .find(|(e, _)| e.aid == aid)
        .context(format!("在section中找不到aid为`{aid}`的ep"))?;

    let tasks = Tasks::new(config, &ep.arc.pic);

    let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if let Some(cid) = cid {
        // 如果有cid，则说明是要下载单个分P
        let Some(page) = ep.pages.iter().find(|p| p.cid == cid) else {
            return Err(anyhow!("找不到cid为`{cid}`的分P"));
        };
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: ep.aid,
            bvid: Some(ep.bvid.clone()),
            cid: page.cid,
            ep_id: None,
            duration: page.duration,
            pub_ts: ep.arc.pubdate,
            collection_title: ugc_season.title.clone(),
            part_title: Some(page.part.clone()),
            part_order: Some(page.page),
            episode_title: ep.title.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        return Ok(vec![progress]);
    }

    if ep.pages.len() == 1 {
        // 如果只有一个分P，则直接创建一个progress
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: ep.aid,
            bvid: Some(ep.bvid.clone()),
            cid: ep.pages[0].cid,
            ep_id: None,
            duration: ep.arc.duration,
            pub_ts: ep.arc.pubdate,
            collection_title: ugc_season.title.clone(),
            part_title: None,
            part_order: None,
            episode_title: ep.title.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            merge_task: tasks.merge,
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        return Ok(vec![progress]);
    }

    // 如果有多个分P，则为每个分P创建一个progress
    let mut progresses = Vec::new();
    for page in &ep.pages {
        let mut progress = DownloadProgress {
            task_id: Uuid::new_v4().to_string(),
            episode_type: EpisodeType::Normal,
            aid: ep.aid,
            bvid: Some(ep.bvid.clone()),
            cid: page.cid,
            ep_id: None,
            duration: page.duration,
            pub_ts: ep.arc.pubdate,
            collection_title: ugc_season.title.clone(),
            part_title: Some(page.part.clone()),
            part_order: Some(page.page),
            episode_title: ep.title.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video.clone(),
            audio_task: tasks.audio.clone(),
            merge_task: tasks.merge.clone(),
            create_ts,
            completed_ts: None,
        };

        progress
            .update_fmt_fields(config)
            .context("更新需要格式化的字段失败")?;

        progresses.push(progress);
    }
    Ok(progresses)
}

struct Tasks {
    video: VideoTask,
    audio: AudioTask,
    merge: MergeTask,
}

impl Tasks {
    fn new(config: &Config, _cover_url: &str) -> Self {
        let video = VideoTask {
            selected: config.download_video,
            url: String::new(),
            video_quality: VideoQuality::Unknown,
            codec_type: CodecType::Unknown,
            content_length: 0,
            chunks: Vec::new(),
            completed: false,
        };

        let audio = AudioTask {
            selected: config.download_audio,
            url: String::new(),
            audio_quality: AudioQuality::Unknown,
            content_length: 0,
            chunks: Vec::new(),
            completed: false,
        };

        let merge = MergeTask {
            selected: config.auto_merge,
            completed: false,
        };

        Self {
            video,
            audio,
            merge,
        }
    }
}
