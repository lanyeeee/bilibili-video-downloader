//! 一个弹幕实例，但是没有位置信息

use super::canvas::CanvasConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DanmakuType {
    #[default]
    Float,
    Top,
    Bottom,
    Reverse,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Danmaku {
    pub timeline_s: f64,
    pub content: String,
    pub r#type: DanmakuType,
    /// 虽然这里有 fontsize，但是我们实际上使用 canvas config 的 font size，
    /// 否在在调节分辨率的时候字体会发生变化。
    pub fontsize: u32,
    pub rgb: (u8, u8, u8),
}

impl Danmaku {
    /// 计算弹幕的“像素长度”，会乘上一个缩放因子
    ///
    /// 汉字算一个全宽，英文算2/3宽
    #[allow(clippy::cast_lossless)]
    pub fn length(&self, config: &CanvasConfig) -> f64 {
        let pts = config.font_size
            * self
                .content
                .chars()
                .map(|ch| if ch.is_ascii() { 2 } else { 3 })
                .sum::<u32>()
            / 3;

        pts as f64 * config.width_ratio
    }
}
