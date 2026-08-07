use std::{
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use eyre::{OptionExt, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::Config,
    downloader::{
        download_task::DownloadTask,
        tasks::{
            audio_task::AudioTask, cover_task::CoverTask, danmaku_task::DanmakuTask,
            json_task::JsonTask, nfo_task::NfoTask, subtitle_task::SubtitleTask,
            video_process_task::VideoProcessTask, video_task::VideoTask,
        },
    },
    events::DownloadEvent,
    extensions::AppHandleExt,
    plugin::hook_context::{
        AfterPrepareContext, BeforeVideoProcessContext, HookContext, OnCompletedContext,
    },
    types::{
        audio_quality::AudioQuality,
        bangumi_info::BangumiInfo,
        cheese_info::CheeseInfo,
        codec_type::CodecType,
        get_normal_info_params::GetNormalInfoParams,
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
    pub episode_title_in_collection: Option<String>,
    pub episode_order: i64,
    pub up_name: Option<String>,
    pub up_uid: Option<i64>,
    pub up_avatar: Option<String>,
    pub episode_dir: PathBuf,
    pub filename: String,
    pub video_task: VideoTask,
    pub audio_task: AudioTask,
    pub video_process_task: VideoProcessTask,
    pub subtitle_task: SubtitleTask,
    pub danmaku_task: DanmakuTask,
    pub cover_task: CoverTask,
    pub nfo_task: NfoTask,
    pub json_task: JsonTask,
    pub create_ts: u64,
    pub completed_ts: Option<u64>,
    pub is_drm: bool,
    pub is_preview: bool,
}

impl DownloadProgress {
    #[instrument(level = "error", skip_all)]
    pub fn from_normal(
        app: &AppHandle,
        info: &NormalInfo,
        aid: i64,
        cid: Option<i64>,
    ) -> eyre::Result<Vec<Self>> {
        let config = app.get_config().read().clone();

        if let Some(ugc_season) = &info.ugc_season {
            create_normal_progresses_for_season(ugc_season, info, aid, cid, &config)
        } else {
            create_normal_progresses_for_single(info, cid, &config)
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    #[instrument(level = "error", skip_all)]
    pub fn from_bangumi(app: &AppHandle, info: &BangumiInfo, ep_id: i64) -> eyre::Result<Self> {
        let (episode, episode_order) = info.get_episode_with_order(ep_id)?;
        let Some(duration) = episode.duration else {
            return Err(eyre!("duration为None"));
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

        let progress = Self {
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
            episode_title_in_collection: None,
            episode_order,
            up_name,
            up_uid,
            up_avatar,
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        Ok(progress)
    }

    #[instrument(level = "error", skip_all)]
    pub fn from_cheese(app: &AppHandle, info: &CheeseInfo, ep_id: i64) -> eyre::Result<Self> {
        let episode = info
            .episodes
            .iter()
            .find(|ep| ep.id == ep_id)
            .ok_or_eyre("找不到ep_id对应的课程")?;

        let config = app.get_config().read().clone();

        let tasks = Tasks::new(&config, &episode.cover);

        let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let progress = Self {
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
            episode_title_in_collection: None,
            episode_order: episode.index,
            up_name: Some(info.up_info.uname.clone()),
            up_uid: Some(info.up_info.mid),
            up_avatar: Some(info.up_info.avatar.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        Ok(progress)
    }

    #[instrument(level = "error", skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn process(&mut self, download_task: &Arc<DownloadTask>) -> eyre::Result<()> {
        let app = &download_task.app;
        let _ = DownloadEvent::ProgressPreparing {
            task_id: self.task_id.clone(),
        }
        .emit(app);

        self.prepare(app).await.wrap_err("准备下载失败")?;

        let progress_before_hook = self.clone();
        app.get_plugin_manager()
            .run_hook(HookContext::AfterPrepare(AfterPrepareContext::new(self)))
            .await?;
        if *self != progress_before_hook {
            download_task.update_progress(|p| *p = self.clone());
        }

        self.completed_ts = None; // 重置完成时间戳
        download_task.update_progress(|p| *p = self.clone());

        std::fs::create_dir_all(&self.episode_dir)
            .wrap_err(format!("创建目录`{}`失败", self.episode_dir.display()))?;

        let mut player_info = None;
        let mut episode_info = None;

        if !self.video_task.is_completed() && self.video_task.content_length != 0 {
            self.video_task
                .process(download_task, self)
                .await
                .wrap_err("下载视频文件失败")?;
            tracing::debug!("视频下载任务完成");
        }

        if !self.audio_task.is_completed() && self.audio_task.content_length != 0 {
            self.audio_task
                .process(download_task, self)
                .await
                .wrap_err("下载音频文件失败")?;
            tracing::debug!("音频下载任务完成");
        }

        let progress_before_hook = self.clone();
        app.get_plugin_manager()
            .run_hook(HookContext::BeforeVideoProcess(
                BeforeVideoProcessContext::new(self),
            ))
            .await?;
        if *self != progress_before_hook {
            download_task.update_progress(|p| *p = self.clone());
        }

        let video_process_task_is_completed = self.video_process_task.is_completed();
        if self.is_drm && !video_process_task_is_completed {
            download_task.update_progress(|p| {
                p.video_process_task.skipped = true;
                p.video_process_task.completed = true;
            });
            tracing::debug!("受版权保护(DRM)，无法处理，已跳过视频处理任务");
        } else if !video_process_task_is_completed {
            self.video_process_task
                .process(download_task, self, &mut player_info)
                .await
                .wrap_err("视频处理失败")?;
            tracing::debug!("视频处理任务完成");
        }

        if !self.danmaku_task.is_completed() {
            self.danmaku_task
                .process(download_task, self)
                .await
                .wrap_err("下载弹幕失败")?;
            tracing::debug!("弹幕下载任务完成");
        }

        if !self.subtitle_task.is_completed() {
            self.subtitle_task
                .process(download_task, self, &mut player_info)
                .await
                .wrap_err("下载字幕失败")?;
            tracing::debug!("字幕下载任务完成");
        }

        if !self.cover_task.is_completed() {
            self.cover_task
                .process(download_task, self)
                .await
                .wrap_err("下载封面失败")?;
            tracing::debug!("封面下载任务完成");
        }

        if !self.nfo_task.is_completed() {
            self.nfo_task
                .process(download_task, self, &mut episode_info)
                .await
                .wrap_err("下载NFO失败")?;
            tracing::debug!("NFO下载任务完成");
        }

        if !self.json_task.is_completed() {
            self.json_task
                .process(download_task, self, &mut episode_info)
                .await
                .wrap_err("下载JSON元数据失败")?;
            tracing::debug!("JSON元数据下载任务完成");
        }

        let completed_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();
        if let Some(completed_ts) = completed_ts {
            self.completed_ts = Some(completed_ts);
            download_task.update_progress(|p| p.completed_ts = Some(completed_ts));
        }

        let progress_before_hook = self.clone();
        app.get_plugin_manager()
            .run_hook(HookContext::OnCompleted(OnCompletedContext::new(self)))
            .await?;
        if *self != progress_before_hook {
            download_task.update_progress(|p| *p = self.clone());
        }

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn prepare(&mut self, app: &AppHandle) -> eyre::Result<()> {
        let video_selected = self.video_task.selected;
        let video_completed = self.video_task.completed;
        let audio_selected = self.audio_task.selected;
        let audio_completed = self.audio_task.completed;

        // 通过合集批量创建的任务时，合集接口只提供视频在合集内的标题
        // 所以在创建任务阶段，episode_title暂时用它占位
        // 在真正开始下载前，需要重新获取视频的真实标题
        if self.episode_type == EpisodeType::Normal && self.episode_title_in_collection.is_some() {
            let bili_client = app.get_bili_client();
            let info = bili_client
                .get_normal_info(GetNormalInfoParams::Aid(self.aid))
                .await
                .wrap_err("获取视频信息失败")?;
            self.episode_title = info.title;
        }

        if (!video_selected && !audio_selected) || (video_completed && audio_completed) {
            // 如果视频和音频都没有选中，或者都已经完成，则更新需要格式化的字段就返回
            self.update_fmt_fields(app)
                .wrap_err("更新需要格式化的字段失败")?;
            return Ok(());
        }

        let bili_client = app.get_bili_client();

        match self.episode_type {
            EpisodeType::Normal => {
                let Some(bvid) = &self.bvid else {
                    return Err(eyre!("progress中的bvid为None，无法获取视频链接"));
                };

                let media_url = bili_client
                    .get_normal_url(bvid, self.cid)
                    .await
                    .wrap_err("获取视频链接失败")?;

                self.is_preview = !media_url.durl.is_empty() && media_url.dash.video.is_empty();

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_normal(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_normal(app, &media_url).await;
                }
            }
            EpisodeType::Bangumi => {
                let media_url = bili_client
                    .get_bangumi_url(self.cid)
                    .await
                    .wrap_err("获取番剧视频链接失败")?;

                self.is_preview = media_url.is_preview != 0;

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_bangumi(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_bangumi(app, &media_url).await;
                }
            }
            EpisodeType::Cheese => {
                let Some(ep_id) = self.ep_id else {
                    return Err(eyre!("progress中的ep_id为None，无法获取课程视频链接"));
                };
                let media_url = bili_client
                    .get_cheese_url(ep_id)
                    .await
                    .wrap_err("获取课程视频链接失败")?;

                self.is_drm = media_url.is_drm;
                self.is_preview = media_url.is_preview != 0;

                if video_selected && !video_completed {
                    // 如果视频被选中且未完成，则准备视频任务
                    self.video_task.prepare_cheese(app, &media_url).await?;
                }

                if audio_selected && !audio_completed {
                    // 如果音频被选中且未完成，则准备音频任务
                    self.audio_task.prepare_cheese(app, &media_url).await;
                }
            }
        }

        self.update_fmt_fields(app)
            .wrap_err("更新需要格式化的字段失败")?;

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    fn update_fmt_fields(&mut self, app: &AppHandle) -> eyre::Result<()> {
        let fmt_params = self.create_fmt_params();

        let config = app.get_config().read().clone();
        let (episode_dir, filename) = fmt_params.get_episode_dir_and_filename(&config)?;

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
            episode_title_in_collection: self.episode_title_in_collection.clone(),
            episode_order: self.episode_order,
            part_title: self.part_title.clone(),
            part_order: self.part_order,
            up_name: self.up_name.clone(),
            up_uid: self.up_uid,
            create_ts: self.create_ts,
            video_quality: self.video_task.video_quality,
            codec_type: self.video_task.codec_type,
            audio_quality: self.audio_task.audio_quality,
        }
    }

    #[instrument(level = "error", skip_all)]
    pub fn save(&self, app: &AppHandle, allow_create: bool) -> eyre::Result<()> {
        let progress = self.clone();
        let file_name = format!("{}.json", progress.task_id);

        let app_data_dir = app.path().app_data_dir()?;
        let tasks_dir = app_data_dir.join(".下载任务");
        std::fs::create_dir_all(&tasks_dir)?;

        let save_path = tasks_dir.join(file_name);
        if !allow_create && !save_path.exists() {
            return Ok(());
        }

        let progress_json = serde_json::to_string(&progress)?;
        std::fs::write(save_path, progress_json)?;

        Ok(())
    }

    pub fn is_completed(&self) -> bool {
        self.video_task.is_completed()
            && self.audio_task.is_completed()
            && self.video_process_task.is_completed()
            && self.danmaku_task.is_completed()
            && self.subtitle_task.is_completed()
            && self.cover_task.is_completed()
            && self.nfo_task.is_completed()
            && self.json_task.is_completed()
    }

    pub fn mark_uncompleted(&mut self) {
        self.video_task.mark_uncompleted();
        self.audio_task.mark_uncompleted();
        self.video_process_task.mark_uncompleted();
        self.danmaku_task.mark_uncompleted();
        self.subtitle_task.mark_uncompleted();
        self.cover_task.mark_uncompleted();
        self.nfo_task.mark_uncompleted();
        self.json_task.mark_uncompleted();
    }
}

#[allow(clippy::too_many_lines)]
#[instrument(level = "error", skip_all)]
fn create_normal_progresses_for_single(
    info: &NormalInfo,
    cid: Option<i64>,
    config: &Config,
) -> eyre::Result<Vec<DownloadProgress>> {
    let tasks = Tasks::new(config, &info.pic);

    let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if let Some(cid) = cid {
        // 如果有cid，则说明是要下载单个分P
        let Some(page) = info.pages.iter().find(|p| p.cid == cid) else {
            return Err(eyre!("找不到cid对应的分P"));
        };
        let progress = DownloadProgress {
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
            episode_title_in_collection: None,
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        return Ok(vec![progress]);
    }

    if info.pages.len() == 1 {
        // 如果只有一个分P，则直接创建一个progress
        let progress = DownloadProgress {
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
            episode_title_in_collection: None,
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        return Ok(vec![progress]);
    }
    // 如果有多个分P，则为每个分P创建一个progress
    let mut progresses = Vec::new();
    for page in &info.pages {
        let progress = DownloadProgress {
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
            episode_title_in_collection: None,
            episode_order: 1,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video.clone(),
            audio_task: tasks.audio.clone(),
            video_process_task: tasks.video_process.clone(),
            danmaku_task: tasks.danmaku.clone(),
            subtitle_task: tasks.subtitle.clone(),
            cover_task: tasks.cover.clone(),
            nfo_task: tasks.nfo.clone(),
            json_task: tasks.json.clone(),
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        progresses.push(progress);
    }
    Ok(progresses)
}

#[allow(clippy::too_many_lines)]
#[instrument(level = "error", skip_all)]
fn create_normal_progresses_for_season(
    ugc_season: &UgcSeason,
    info: &NormalInfo,
    aid: i64,
    cid: Option<i64>,
    config: &Config,
) -> eyre::Result<Vec<DownloadProgress>> {
    let section_index = ugc_season
        .sections
        .iter()
        .position(|s| s.episodes.iter().any(|e| e.aid == aid))
        .ok_or_eyre("找不到含有对应aid的section")?;
    let section = &ugc_season.sections[section_index];
    #[allow(clippy::cast_possible_wrap)]
    let (ep, episode_order) = section
        .episodes
        .iter()
        .enumerate()
        .map(|(i, e)| (e, i as i64 + 1))
        .find(|(e, _)| e.aid == aid)
        .ok_or_eyre("在section中找不到aid对应的ep")?;

    let tasks = Tasks::new(config, &ep.arc.pic);

    // 合集接口返回的只有在合集内的标题，视频标题先用它占位，到prepare时再按aid获取真实标题
    let episode_title = ep.title.clone();
    let episode_title_in_collection = Some(ep.title.clone());

    let create_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if let Some(cid) = cid {
        // 如果有cid，则说明是要下载单个分P
        let Some(page) = ep.pages.iter().find(|p| p.cid == cid) else {
            return Err(eyre!("找不到cid对应的分P"));
        };
        let progress = DownloadProgress {
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
            episode_title: episode_title.clone(),
            episode_title_in_collection: episode_title_in_collection.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        return Ok(vec![progress]);
    }

    if ep.pages.len() == 1 {
        // 如果只有一个分P，则直接创建一个progress
        let progress = DownloadProgress {
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
            episode_title: episode_title.clone(),
            episode_title_in_collection: episode_title_in_collection.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video,
            audio_task: tasks.audio,
            video_process_task: tasks.video_process,
            danmaku_task: tasks.danmaku,
            subtitle_task: tasks.subtitle,
            cover_task: tasks.cover,
            nfo_task: tasks.nfo,
            json_task: tasks.json,
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        return Ok(vec![progress]);
    }

    // 如果有多个分P，则为每个分P创建一个progress
    let mut progresses = Vec::new();
    for page in &ep.pages {
        let progress = DownloadProgress {
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
            episode_title: episode_title.clone(),
            episode_title_in_collection: episode_title_in_collection.clone(),
            episode_order,
            up_name: Some(info.owner.name.clone()),
            up_uid: Some(info.owner.mid),
            up_avatar: Some(info.owner.face.clone()),
            episode_dir: PathBuf::new(),
            filename: String::new(),
            video_task: tasks.video.clone(),
            audio_task: tasks.audio.clone(),
            video_process_task: tasks.video_process.clone(),
            danmaku_task: tasks.danmaku.clone(),
            subtitle_task: tasks.subtitle.clone(),
            cover_task: tasks.cover.clone(),
            nfo_task: tasks.nfo.clone(),
            json_task: tasks.json.clone(),
            create_ts,
            completed_ts: None,
            is_drm: false,
            is_preview: false,
        };

        progresses.push(progress);
    }
    Ok(progresses)
}

struct Tasks {
    video: VideoTask,
    audio: AudioTask,
    video_process: VideoProcessTask,
    danmaku: DanmakuTask,
    subtitle: SubtitleTask,
    cover: CoverTask,
    nfo: NfoTask,
    json: JsonTask,
}

impl Tasks {
    fn new(config: &Config, cover_url: &str) -> Self {
        let video = VideoTask {
            selected: config.download_video,
            url: String::new(),
            video_quality: VideoQuality::Unknown,
            codec_type: CodecType::Unknown,
            content_length: 0,
            chunks: Vec::new(),
            completed: false,
            skipped: false,
        };

        let audio = AudioTask {
            selected: config.download_audio,
            url: String::new(),
            audio_quality: AudioQuality::Unknown,
            content_length: 0,
            chunks: Vec::new(),
            completed: false,
            skipped: false,
        };

        let video_process = VideoProcessTask {
            merge_selected: config.auto_merge,
            embed_chapter_selected: config.embed_chapter,
            embed_skip_selected: config.embed_skip,
            completed: false,
            skipped: false,
        };

        let danmaku = DanmakuTask {
            xml_selected: config.download_xml_danmaku,
            ass_selected: config.download_ass_danmaku,
            json_selected: config.download_json_danmaku,
            completed: false,
            skipped: false,
        };

        let subtitle = SubtitleTask {
            selected: config.download_subtitle,
            completed: false,
        };

        let cover = CoverTask {
            selected: config.download_cover,
            url: cover_url.to_string(),
            completed: false,
        };

        let nfo = NfoTask {
            selected: config.download_nfo,
            completed: false,
            skipped: false,
        };

        let json = JsonTask {
            selected: config.download_json,
            completed: false,
        };

        Self {
            video,
            audio,
            video_process,
            danmaku,
            subtitle,
            cover,
            nfo,
            json,
        }
    }
}
