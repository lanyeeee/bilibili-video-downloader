//! 决定绘画策略

use float_ord::FloatOrd;
use lane::{Collision, Lane};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    danmaku::Danmaku,
    drawable::{DrawEffect, Drawable},
};
mod lane;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CanvasConfig {
    /// 弹幕在屏幕上的【持续时间】，单位为秒，可以有小数
    pub duration: f64,
    /// 渲染的屏幕分辨率，这个并不会影响渲染区域的大小，只是字体的相对大小，可以不用改动
    pub width: u32,
    /// 渲染的屏幕分辨率，这个并不会影响渲染区域的大小，只是字体的相对大小，可以不用改动
    pub height: u32,
    /// 使用字体名称
    pub font: String,
    /// 弹幕字体大小
    pub font_size: u32,
    /// 是一个比例数，用来计算平衡不同字体的宽度
    /// 有的字体比较粗、比较宽，可以适当调大（如 1.4、1.6）
    /// 有的字体比较细、比较窄，可以适当调小（如 1.0、1.2）
    pub width_ratio: f64,
    /// 用来调整弹幕时间的水平距离，单位是像素，如果想拉开弹幕之间的距离，可以调大
    pub horizontal_gap: f64,
    /// 计算弹幕高度的数值，即【行高度/行间距】。数值越大，弹幕的垂直距离越大
    pub lane_size: u32,
    /// 【正常弹幕的屏幕填充占比】，默认为 50%，即“半屏填充”。
    pub float_percentage: f64,
    /// 屏幕上底部弹幕最多高度百分比
    #[serde(skip)]
    pub bottom_percentage: f64,
    /// 弹幕的不透明度，越小越透明，越大越不透明
    pub alpha: f64,
    /// 字体是否加粗
    pub bold: bool,
    /// 弹幕的描边宽度，单位为像素
    pub outline: f64,
    /// 弹幕时间轴偏移，>0 会让弹幕延后，<0 会让弹幕提前，单位为秒
    pub time_offset: f64,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        CanvasConfig {
            duration: 15.0,
            width: 1280,
            height: 720,
            font: "黑体".to_string(),
            font_size: 25,
            width_ratio: 1.2,
            horizontal_gap: 20.0,
            lane_size: 32,
            float_percentage: 0.5,
            bottom_percentage: 0.3,
            alpha: 0.7,
            bold: false,
            outline: 0.8,
            time_offset: 0.0,
        }
    }
}

impl CanvasConfig {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
    #[allow(clippy::cast_sign_loss)]
    pub fn canvas(self) -> Canvas {
        let float_lanes_cnt =
            (self.float_percentage * self.height as f64 / self.lane_size as f64) as usize;
        let bottom_lanes_cnt =
            (self.bottom_percentage * self.height as f64 / self.lane_size as f64) as usize;

        Canvas {
            config: self,
            float_lanes: vec![None; float_lanes_cnt],
            bottom_lanes: vec![None; bottom_lanes_cnt],
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn get_opacity(&self) -> u8 {
        255 - (self.alpha * 255.) as u8
    }
}

pub struct Canvas {
    pub config: CanvasConfig,
    pub float_lanes: Vec<Option<Lane>>,
    #[allow(dead_code)]
    pub bottom_lanes: Vec<Option<Lane>>,
}

impl Canvas {
    pub fn draw(&mut self, mut danmaku: Danmaku) -> Option<Drawable> {
        use super::danmaku::DanmakuType::{Bottom, Float, Reverse, Top};
        danmaku.timeline_s += self.config.time_offset;
        if danmaku.timeline_s < 0.0 {
            return None;
        }
        match danmaku.r#type {
            Float => self.draw_float(danmaku),
            Bottom | Top | Reverse => {
                // 不喜欢底部弹幕，直接转成 Float
                // 这是 feature 不是 bug
                danmaku.r#type = Float;
                self.draw_float(danmaku)
            }
        }
    }

    fn draw_float(&mut self, mut danmaku: Danmaku) -> Option<Drawable> {
        let mut collisions = Vec::with_capacity(self.float_lanes.len());
        for (idx, lane) in self.float_lanes.iter_mut().enumerate() {
            match lane {
                // 优先画不存在的槽位
                None => {
                    return Some(self.draw_float_in_lane(danmaku, idx));
                }
                Some(l) => {
                    let col = l.available_for(&danmaku, &self.config);
                    match col {
                        Collision::Separate { .. } | Collision::NotEnoughTime { .. } => {
                            return Some(self.draw_float_in_lane(danmaku, idx));
                        }
                        Collision::Collide { time_needed } => {
                            collisions.push((FloatOrd(time_needed), idx));
                        }
                    }
                }
            }
        }
        // 允许部分弹幕在延迟后填充
        if !collisions.is_empty() {
            collisions.sort_unstable();
            let (FloatOrd(time_need), lane_idx) = collisions[0];
            if time_need < 1.0 {
                // 只允许延迟 1s
                danmaku.timeline_s += time_need + 0.01; // 间隔也不要太小了
                return Some(self.draw_float_in_lane(danmaku, lane_idx));
            }
        }
        None
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    fn draw_float_in_lane(&mut self, danmaku: Danmaku, lane_idx: usize) -> Drawable {
        self.float_lanes[lane_idx] = Some(Lane::draw(&danmaku, &self.config));
        let y = lane_idx as i32 * self.config.lane_size as i32;
        let l = danmaku.length(&self.config);
        Drawable::new(
            danmaku,
            self.config.duration,
            "Float",
            DrawEffect::Move {
                start: (self.config.width as i32, y),
                end: (-(l as i32), y),
            },
        )
    }
}
