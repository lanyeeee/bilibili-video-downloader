pub mod ass_writer;
pub mod canvas;
pub mod danmaku;
pub mod drawable;

use std::{cmp::Ordering, fs::File};

use anyhow::anyhow;
use ass_writer::AssWriter;
use canvas::CanvasConfig;
use danmaku::{Danmaku, DanmakuType};
use yaserde::{YaDeserialize, YaSerialize};

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "d")]
pub struct DamakuXmlDTag {
    #[yaserde(attribute = true)]
    pub p: String,
    #[yaserde(text = true)]
    pub body: Option<String>,
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "i")]
pub struct DanmakuXmlITag {
    pub chatid: i64,
    #[yaserde(rename = "d")]
    pub elems: Vec<DamakuXmlDTag>,
}

pub fn xml_to_ass(
    xml: &str,
    ass_file: File,
    title: String,
    config: CanvasConfig,
) -> anyhow::Result<()> {
    let mut writer = AssWriter::new(ass_file, title, config.clone())?;
    let mut canvas = config.canvas();

    let mut danmakus: Vec<Danmaku> = xml_to_danmakus(xml)?;
    danmakus.sort_by(|a, b| {
        a.timeline_s
            .partial_cmp(&b.timeline_s)
            .unwrap_or(Ordering::Equal)
    });

    for danmaku in danmakus {
        if let Some(drawable) = canvas.draw(danmaku) {
            writer.write(drawable)?;
        }
    }

    Ok(())
}

trait ToDanmakuType {
    fn to_danmaku_type(&self) -> anyhow::Result<DanmakuType>;
}

impl ToDanmakuType for u32 {
    fn to_danmaku_type(&self) -> anyhow::Result<DanmakuType> {
        match self {
            1 => Ok(DanmakuType::Float),
            4 => Ok(DanmakuType::Bottom),
            5 => Ok(DanmakuType::Top),
            6 => Ok(DanmakuType::Reverse),
            _ => Err(anyhow!("未知的弹幕类型：{self}")),
        }
    }
}

pub fn xml_to_danmakus(xml: &str) -> anyhow::Result<Vec<Danmaku>> {
    let xml = sanitize_xml(xml);
    let i_tag: DanmakuXmlITag = yaserde::de::from_str(&xml).map_err(|e| anyhow!(e))?;

    let mut danmakus = Vec::new();

    for elem in i_tag.elems {
        let Some(content) = elem.body else {
            continue;
        };

        let mut p_attr = elem.p.split(',');

        let Some(timeline_s) = p_attr.next().and_then(|s| s.parse::<f64>().ok()) else {
            return Err(anyhow!("弹幕`{content}`的p属性中没有时间"));
        };

        let Some(r#type) = p_attr
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .and_then(|num| num.to_danmaku_type().ok())
        else {
            return Err(anyhow!("弹幕`{content}`的p属性中没有弹幕类型"));
        };

        let Some(fontsize) = p_attr.next().and_then(|s| s.parse::<u32>().ok()) else {
            return Err(anyhow!("弹幕`{content}`的p属性中没有字体大小"));
        };

        let Some(rgb) = p_attr.next().and_then(|s| s.parse::<u32>().ok()) else {
            return Err(anyhow!("弹幕`{content}`的p属性中没有颜色"));
        };

        // rgb 是个数字，类似 0x010203
        let r = (rgb >> 16) & 0xff;
        let g = (rgb >> 8) & 0xff;
        let b = rgb & 0xff;

        let danmaku = Danmaku {
            timeline_s,
            content,
            r#type,
            fontsize,
            rgb: (r as u8, g as u8, b as u8),
        };

        danmakus.push(danmaku);
    }

    Ok(danmakus)
}

fn sanitize_xml(s: &str) -> String {
    fn is_valid_xml_char(c: char) -> bool {
        matches!(c,
            '\u{0009}' |
            '\u{000A}' |
            '\u{000D}' |
            '\u{0020}'..='\u{D7FF}' |
            '\u{E000}'..='\u{FFFD}' |
            '\u{10000}'..='\u{10FFFF}'
        )
    }
    s.chars().filter(|&c| is_valid_xml_char(c)).collect()
}
