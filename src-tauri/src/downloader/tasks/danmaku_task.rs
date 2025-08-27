use std::{fs::File, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    danmaku_xml_to_ass::xml_to_ass,
    downloader::{download_progress::DownloadProgress, download_task::DownloadTask},
    extensions::AppHandleExt,
    utils::ToXml,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
pub struct DanmakuTask {
    pub xml_selected: bool,
    pub ass_selected: bool,
    pub json_selected: bool,
    pub completed: bool,
}

impl DanmakuTask {
    pub fn is_completed(&self) -> bool {
        !self.xml_selected && !self.ass_selected && !self.json_selected || self.completed
    }

    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> anyhow::Result<()> {
        let danmaku_task = &progress.danmaku_task;
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let bili_client = download_task.app.get_bili_client();
        let replies = bili_client
            .get_danmaku(progress.aid, progress.cid, progress.duration)
            .await
            .context("获取弹幕失败")?;

        let xml = replies
            .to_xml(progress.cid)
            .context("将弹幕转换为XML失败")?;

        if danmaku_task.xml_selected {
            let xml_path = episode_dir.join(format!("{filename}.弹幕.xml"));
            std::fs::write(&xml_path, &xml)
                .context(format!("保存弹幕XML到`{}`失败", xml_path.display()))?;
        }

        if danmaku_task.ass_selected {
            let config = download_task.app.get_config().read().danmaku_config.clone();
            let ass_path = episode_dir.join(format!("{filename}.弹幕.ass"));
            let ass_file = File::create(&ass_path)
                .context(format!("创建弹幕ASS文件`{}`失败", ass_path.display()))?;
            let title = filename.to_string();
            xml_to_ass(&xml, ass_file, title, config).context("将弹幕XML转换为ASS失败")?;
        }

        if danmaku_task.json_selected {
            let json_path = episode_dir.join(format!("{filename}.弹幕.json"));
            let json_string = serde_json::to_string(&replies).context("将弹幕转换为JSON失败")?;
            std::fs::write(&json_path, json_string)
                .context(format!("保存弹幕JSON到`{}`失败", json_path.display()))?;
        }

        download_task.update_progress(|p| p.danmaku_task.completed = true);

        Ok(())
    }
}
