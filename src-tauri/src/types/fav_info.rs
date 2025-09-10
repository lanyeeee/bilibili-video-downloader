use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct FavInfo {
    pub info: Info,
    pub medias: Option<Vec<MediaInFav>>,
    pub has_more: bool,
    pub ttl: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct Info {
    pub id: i64,
    pub fid: i64,
    pub mid: i64,
    pub attr: i64,
    pub title: String,
    pub cover: String,
    pub upper: Upper,
    pub cover_type: i64,
    pub cnt_info: CntInfo,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub intro: String,
    pub ctime: i64,
    pub mtime: i64,
    pub state: i64,
    pub fav_state: i64,
    pub like_state: i64,
    pub media_count: i64,
    pub is_top: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Upper {
    pub mid: i64,
    pub name: String,
    pub face: String,
    pub followed: bool,
    pub vip_type: i64,
    pub vip_statue: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CntInfo {
    pub collect: i64,
    pub play: i64,
    pub thumb_up: i64,
    pub share: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct MediaInFav {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: i64,
    pub duration: u64,
    pub upper: UpperInMedia,
    pub attr: i64,
    pub cnt_info: CntInfoInMedia,
    pub link: String,
    pub ctime: i64,
    pub pubtime: i64,
    pub fav_time: i64,
    pub bv_id: String,
    pub bvid: String,
    pub ugc: Option<Ugc>,
    pub media_list_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct UpperInMedia {
    pub mid: i64,
    pub name: String,
    pub face: String,
    pub jump_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CntInfoInMedia {
    pub collect: i64,
    pub play: i64,
    pub danmaku: i64,
    pub vt: i64,
    pub play_switch: i64,
    pub reply: i64,
    pub view_text_1: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Ugc {
    pub first_cid: i64,
}
