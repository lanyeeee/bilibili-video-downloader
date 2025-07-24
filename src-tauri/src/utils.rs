use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use anyhow::{anyhow, Context};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    danmaku_xml_to_ass::{DamakuXmlDTag, DanmakuXmlITag},
    protobuf::DmSegMobileReply,
};

pub fn filename_filter(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' | '/' | '\n' => ' ',
            ':' => '：',
            '*' => '⭐',
            '?' => '？',
            '"' => '\'',
            '<' => '《',
            '>' => '》',
            '|' => '丨',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_string()
}

enum BoxSizeField {
    SizeExtendToEnd,
    LargeSize,
    NormalSize(u32),
}

impl From<u32> for BoxSizeField {
    fn from(size: u32) -> Self {
        match size {
            0 => BoxSizeField::SizeExtendToEnd,
            1 => BoxSizeField::LargeSize,
            _ => BoxSizeField::NormalSize(size),
        }
    }
}

pub fn is_mp4_complete(file_path: &Path) -> anyhow::Result<bool> {
    let file = File::open(file_path).context(format!("打开文件`{}`失败", file_path.display()))?;
    let real_size = file
        .metadata()
        .context(format!("获取文件`{}`元数据失败", file_path.display()))?
        .len();
    let mut reader = BufReader::new(file);
    let mut total_size: u64 = 0;

    let mut has_moov_box = false;
    let mut is_first_box = true;

    loop {
        // 读取Box尺寸字段
        let box_size_field: BoxSizeField = match reader.read_u32::<BigEndian>() {
            Ok(s) => s.into(),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // 正常结束
            Err(e) => return Err(anyhow!(e)),
        };
        // 读取Box类型字段
        let mut box_type_bytes = [0u8; 4];
        if let Err(e) = reader.read_exact(&mut box_type_bytes) {
            // 如果在读取type时就结束了，说明文件在box header中间被截断
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(false);
            }
            return Err(anyhow!(e));
        }
        // 如果是第一个Box，检查是否是 'ftyp' Box
        if is_first_box {
            if &box_type_bytes != b"ftyp" {
                // 如果第一个box不是 'ftyp'，则认为它不是一个标准的MP4文件
                return Ok(false);
            }
            is_first_box = false;
        }
        // 检查是否有 'moov' Box
        if &box_type_bytes == b"moov" {
            has_moov_box = true;
        }

        // 获取Box尺寸
        let box_size = match box_size_field {
            // Box延伸到文件末尾，直接返回true
            BoxSizeField::SizeExtendToEnd => return Ok(true),
            BoxSizeField::LargeSize => {
                // 对于尺寸非常大的Box(大于4GB)，其真实的尺寸是一个64位的整数，紧跟在类型字段后面
                let large_box_size = reader.read_u64::<BigEndian>()?;
                // 头部总共16字节(box_size 4 + box_type 4 + large_box_size 8)
                if large_box_size < 16 {
                    // 如果连16字节都不够，说明有问题
                    return Ok(false);
                }
                // 跳过Box剩余的部分
                #[allow(clippy::cast_possible_wrap)]
                reader.seek_relative((large_box_size - 16) as i64)?;
                large_box_size
            }
            BoxSizeField::NormalSize(box_size) => {
                // 头部总共8字节 (size 4 + type 4)
                if box_size < 8 {
                    // 如果连8字节都不够，说明有问题
                    return Ok(false);
                }
                // 跳过Box剩余的部分
                reader.seek_relative(i64::from(box_size - 8))?;

                u64::from(box_size)
            }
        };

        total_size += box_size;

        if total_size > real_size {
            // 如果总大小超过了实际文件大小，说明有问题
            return Ok(false);
        }
    }

    Ok(real_size == total_size && has_moov_box)
}

pub trait ToXml {
    fn to_xml(&self, cid: i64) -> anyhow::Result<String>;
}

impl ToXml for Vec<DmSegMobileReply> {
    fn to_xml(&self, cid: i64) -> anyhow::Result<String> {
        let elems = self
            .iter()
            .flat_map(|reply| &reply.elems)
            .map(|elem| DamakuXmlDTag {
                p: format!(
                    "{},{},{},{},{},{},{},{}",
                    elem.progress / 1000,
                    elem.mode,
                    elem.fontsize,
                    elem.color,
                    elem.ctime,
                    elem.pool,
                    elem.mid_hash.clone(),
                    elem.id_str.clone(),
                ),
                body: Some(elem.content.clone()),
            })
            .collect();

        let i_tag = DanmakuXmlITag { chatid: cid, elems };

        let xml = yaserde::ser::to_string(&i_tag).map_err(|e| anyhow!(e))?;

        Ok(xml)
    }
}
