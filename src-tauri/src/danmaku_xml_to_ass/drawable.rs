//! 可以绘制的实体

use super::danmaku::Danmaku;

/// 弹幕开始绘制的时间就是 danmaku 的时间
pub struct Drawable {
    pub danmaku: Danmaku,
    /// 弹幕一共绘制的时间  
    pub duration: f64,
    /// 弹幕的绘制 style
    pub style_name: &'static str,
    /// 绘制的“特效”
    pub effect: DrawEffect,
}
impl Drawable {
    pub fn new(
        danmaku: Danmaku,
        duration: f64,
        style_name: &'static str,
        effect: DrawEffect,
    ) -> Self {
        Drawable {
            danmaku,
            duration,
            style_name,
            effect,
        }
    }
}

pub enum DrawEffect {
    Move {
        start: (i32, i32),
        end: (i32, i32),
    },
    #[allow(dead_code)]
    Fixed {},
}
