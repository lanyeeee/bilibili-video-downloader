pub struct ChapterSegments {
    pub segments: Vec<ChapterSegment>,
}

#[derive(Clone)]
pub struct ChapterSegment {
    pub title: String,
    pub start: i64,
    pub end: i64,
}

impl ChapterSegments {
    /// 插入一个新的章节片段
    ///
    /// 此函数会处理新片段与现有片段的重叠情况：
    /// - 对于与新片段重叠的现有片段，会将其分割为非重叠的部分
    /// - 新片段会替换所有重叠区域
    /// - 最终结果会按开始时间排序
    ///
    /// # 参数
    /// * `new_segment` - 要插入的新章节片段
    ///
    /// # 示例
    /// ```
    /// // 假设现有片段: [0-10], [20-30]
    /// // 插入新片段: [5-25]
    /// // 结果: [0-5], [5-25], [25-30]
    /// ```
    pub fn insert(&mut self, new_segment: ChapterSegment) {
        // 创建一个新的 Vec 来存储处理后的片段
        // 预分配容量为当前片段数量 + 2，因为最坏情况下每个现有片段可能被分割成两部分，再加上新片段
        let mut processed_segments = Vec::with_capacity(self.segments.len() + 2);

        for segment in &self.segments {
            if !Self::overlaps(segment, &new_segment) {
                // 如果当前片段与新片段没有重叠，直接将当前片段添加到结果中
                processed_segments.push(segment.clone());
                continue;
            }
            // 如果有重叠，需要分割当前片段，只保留不与新片段重叠的部分

            // 处理左侧部分：当前片段开始到新片段开始之间的部分
            // left_end 是左侧部分的结束时间，取当前片段结束时间和新片段开始时间的较小值
            let left_end = segment.end.min(new_segment.start);
            if segment.start < left_end {
                // 只有当左侧部分确实存在时（start < end）才添加
                processed_segments.push(ChapterSegment {
                    title: segment.title.clone(),
                    start: segment.start,
                    end: left_end,
                });
            }

            // 处理右侧部分：新片段结束到当前片段结束之间的部分
            // right_start 是右侧部分的开始时间，取当前片段开始时间和新片段结束时间的较大值
            let right_start = segment.start.max(new_segment.end);
            if right_start < segment.end {
                // 只有当右侧部分确实存在时（start < end）才添加
                processed_segments.push(ChapterSegment {
                    title: segment.title.clone(),
                    start: right_start,
                    end: segment.end,
                });
            }
        }

        // 遍历完所有现有片段并处理完所有重叠后，将新的片段添加到结果列表中
        processed_segments.push(new_segment);

        processed_segments.sort_by(|a, b| a.start.cmp(&b.start));

        self.segments = processed_segments;
    }

    pub fn generate_chapter_metadata(&self, video_duration: u64) -> String {
        use std::fmt::Write;

        fn write_segment(content: &mut String, title: &str, start: i64, end: i64) {
            let _ = writeln!(
                content,
                "[CHAPTER]\nTIMEBASE=1/1\nSTART={start}\nEND={end}\ntitle={title}\n"
            );
        }

        let video_duration = i64::try_from(video_duration).unwrap_or(i64::MAX);

        let mut metadata_content = ";FFMETADATA1\n".to_string();

        let mut last_end = 0;
        for segment in &self.segments {
            // 检查当前片段的开始时间与上一个片段的结束时间之间是否有间隙
            if segment.start > last_end {
                // 如果有间隙，则插入一个标题为空格的空白片段
                write_segment(&mut metadata_content, " ", last_end, segment.start);
            }

            // 写入当前片段
            write_segment(
                &mut metadata_content,
                &segment.title,
                segment.start,
                segment.end,
            );

            // 更新上一个片段的结束时间
            last_end = segment.end;
        }

        // 循环结束后，检查最后一个片段的结尾与视频总时长之间是否还有间隙
        if video_duration > last_end {
            // 如果有，则填充从 last_end 到视频结尾的剩余部分
            write_segment(&mut metadata_content, " ", last_end, video_duration);
        }

        metadata_content
    }

    /// 检查两个片段是否重叠。
    fn overlaps(s1: &ChapterSegment, s2: &ChapterSegment) -> bool {
        s1.start < s1.end && s2.start < s2.end && s1.start < s2.end && s2.start < s1.end
    }
}
