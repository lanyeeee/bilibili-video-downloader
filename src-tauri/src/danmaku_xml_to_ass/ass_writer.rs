use anyhow::Result;
use std::borrow::Cow;
use std::fmt;
use std::io::{BufWriter, Write};

use super::canvas::CanvasConfig;
use super::drawable::{DrawEffect, Drawable};

struct TimePoint {
    t: f64,
}
impl fmt::Display for TimePoint {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_lossless)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.t.floor() as u32;
        let hour = secs / 3600;
        let minutes = (secs % 3600) / 60;

        let left = self.t - (hour * 3600) as f64 - (minutes * 60) as f64;

        write!(f, "{hour}:{minutes:02}:{left:05.2}")
    }
}

struct AssEffect {
    effect: DrawEffect,
}
impl fmt::Display for AssEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.effect {
            DrawEffect::Move { start, end } => {
                let (x0, y0) = start;
                let (x1, y1) = end;
                write!(f, "\\move({x0}, {y0}, {x1}, {y1})")
            }
            DrawEffect::Fixed {} => fmt::Result::Err(fmt::Error),
        }
    }
}

impl CanvasConfig {
    #[allow(clippy::cast_lossless)]
    pub fn ass_styles(&self) -> Vec<String> {
        let opacity = self.get_opacity();
        vec![
            // Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, \
            // Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, \
            // Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
            format!(
                "Style: Float,{font},{font_size},&H{a:02x}FFFFFF,&H00FFFFFF,&H{a:02x}000000,&H00000000,\
                {bold}, 0, 0, 0, 100, 100, 0.00, 0.00, 1, \
                {outline}, 0, 7, 0, 0, 0, 1",
                a = opacity,
                font = self.font,
                font_size = self.font_size,
                bold = self.bold as u8,
                outline = self.outline,
            ),
            format!(
                "Style: Bottom,{font},{font_size},&H{a:02x}FFFFFF,&H00FFFFFF,&H{a:02x}000000,&H00000000,\
                {bold}, 0, 0, 0, 100, 100, 0.00, 0.00, 1, \
                {outline}, 0, 7, 0, 0, 0, 1",
                a = opacity,
                font = self.font,
                font_size = self.font_size,
                bold = self.bold as u8,
                outline = self.outline,
            ),
            format!(
                "Style: Top,{font},{font_size},&H{a:02x}FFFFFF,&H00FFFFFF,&H{a:02x}000000,&H00000000,\
                {bold}, 0, 0, 0, 100, 100, 0.00, 0.00, 1, \
                {outline}, 0, 7, 0, 0, 0, 1",
                a = opacity,
                font = self.font,
                font_size = self.font_size,
                bold = self.bold as u8,
                outline = self.outline,
            ),
        ]
    }
}

struct CanvasStyles(Vec<String>);
impl fmt::Display for CanvasStyles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for style in &self.0 {
            writeln!(f, "{style}")?;
        }
        Ok(())
    }
}

pub struct AssWriter<W: Write> {
    f: BufWriter<W>,
    title: String,
    canvas_config: CanvasConfig,
}

impl<W: Write> AssWriter<W> {
    pub fn new(f: W, title: String, canvas_config: CanvasConfig) -> Result<Self> {
        let mut this = AssWriter {
            // 对于 HDD、docker 之类的场景，磁盘 IO 是非常大的瓶颈。使用大缓存
            f: BufWriter::with_capacity(10 << 20, f),
            title,
            canvas_config,
        };

        this.init()?;

        Ok(this)
    }

    pub fn init(&mut self) -> Result<()> {
        write!(
            self.f,
            "\
            [Script Info]\n\
            ; Script generated by bilibili-video-downloader (https://github.com/lanyeeee/bilibili-video-downloader)\n\
            Title: {title}\n\
            Script Updated By: bilibili-video-downloader (https://github.com/lanyeeee/bilibili-video-downloader)\n\
            ScriptType: v4.00+\n\
            PlayResX: {width}\n\
            PlayResY: {height}\n\
            Aspect Ratio: {width}:{height}\n\
            Collisions: Normal\n\
            WrapStyle: 2\n\
            ScaledBorderAndShadow: yes\n\
            YCbCr Matrix: TV.601\n\
            \n\
            \n\
            [V4+ Styles]\n\
            Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, \
                    Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, \
                    Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
            {styles}\
            \n\
            [Events]\n\
            Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
            ",
            title = self.title,
            width = self.canvas_config.width,
            height = self.canvas_config.height,
            styles = CanvasStyles(self.canvas_config.ass_styles()),
        )?;
        Ok(())
    }

    pub fn write(&mut self, drawable: Drawable) -> Result<()> {
        writeln!(
            self.f,
            // Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
            "Dialogue: 2,{start},{end},{style},,0,0,0,,{{{effect}\\c&H{b:02x}{g:02x}{r:02x}&}}{text}",
            start = TimePoint {
                t: drawable.danmaku.timeline_s
            },
            end = TimePoint {
                t: drawable.danmaku.timeline_s + drawable.duration
            },
            style = drawable.style_name,
            effect = AssEffect {
                effect: drawable.effect
            },
            b = drawable.danmaku.rgb.2,
            g = drawable.danmaku.rgb.1,
            r = drawable.danmaku.rgb.0,
            text = escape_text(&drawable.danmaku.content),
        )?;
        Ok(())
    }
}

fn escape_text(text: &str) -> Cow<str> {
    let text = text.trim();
    if memchr::memchr(b'\n', text.as_bytes()).is_some() {
        Cow::from(text.replace('\n', "\\N"))
    } else {
        Cow::from(text)
    }
}
