use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalInfo {
    pub bvid: String,
    pub aid: i64,
    pub videos: i64,
    pub tid: i64,
    pub tid_v2: i64,
    pub tname: String,
    pub tname_v2: String,
    pub copyright: i64,
    pub pic: String,
    pub title: String,
    pub pubdate: i64,
    pub ctime: i64,
    pub desc: String,
    pub desc_v2: Option<Vec<DescV2>>,
    pub state: i64,
    pub duration: u64,
    pub rights: Rights,
    pub owner: OwnerInNormal,
    pub stat: StatInNormal,
    pub argue_info: ArgueInfo,
    pub dynamic: String,
    pub cid: i64,
    pub dimension: Dimension,
    pub teenage_mode: i64,
    pub is_chargeable_season: bool,
    pub is_story: bool,
    pub is_upower_exclusive: bool,
    pub is_upower_play: bool,
    pub is_upower_preview: bool,
    pub enable_vt: i64,
    pub vt_display: String,
    pub is_upower_exclusive_with_qa: bool,
    pub no_cache: bool,
    pub pages: Vec<PageInNormal>,
    pub subtitle: SubtitleInNormal,
    pub staff: Option<Vec<Staff>>,
    pub ugc_season: Option<UgcSeason>,
    pub is_season_display: bool,
    pub user_garb: UserGarb,
    pub honor_reply: HonorReply,
    pub like_icon: String,
    pub need_jump_bv: bool,
    pub disable_show_up_info: bool,
    pub is_story_play: i64,
    pub is_view_self: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct DescV2 {
    pub raw_text: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub biz_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Rights {
    pub bp: i64,
    pub elec: i64,
    pub download: i64,
    pub movie: i64,
    pub pay: i64,
    pub hd5: i64,
    pub no_reprint: i64,
    pub autoplay: i64,
    pub ugc_pay: i64,
    pub is_cooperation: i64,
    pub ugc_pay_preview: i64,
    pub no_background: i64,
    pub clean_mode: i64,
    pub is_stein_gate: i64,
    pub is_360: i64,
    pub no_share: i64,
    pub arc_pay: i64,
    pub free_watch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct OwnerInNormal {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct StatInNormal {
    pub aid: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub favorite: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub dislike: i64,
    pub evaluation: String,
    pub vt: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct ArgueInfo {
    pub argue_msg: String,
    pub argue_type: i64,
    pub argue_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Dimension {
    pub width: i64,
    pub height: i64,
    pub rotate: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
#[allow(clippy::struct_field_names)]
pub struct PageInNormal {
    pub cid: i64,
    pub page: i64,
    pub from: String,
    pub part: String,
    pub duration: u64,
    pub vid: String,
    pub weblink: String,
    pub dimension: Dimension,
    pub ctime: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SubtitleInNormal {
    pub allow_submit: bool,
    pub list: Vec<SubtitleDetailInNormal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SubtitleDetailInNormal {
    pub id: i64,
    pub lan: String,
    pub lan_doc: String,
    pub is_lock: bool,
    pub subtitle_url: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub id_str: String,
    pub ai_type: i64,
    pub ai_status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct UserGarb {
    pub url_image_ani_cut: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct HonorReply {
    pub honor: Option<Vec<Honor>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Honor {
    pub aid: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub desc: String,
    pub weekly_recommend_num: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct UgcSeason {
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub mid: i64,
    pub intro: String,
    pub sign_state: i64,
    pub attribute: i64,
    pub sections: Vec<SectionInNormal>,
    pub stat: StatInNormalSeason,
    pub ep_count: i64,
    pub season_type: i64,
    pub is_pay_season: bool,
    pub enable_vt: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SectionInNormal {
    pub season_id: i64,
    pub id: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub episodes: Vec<EpInNormal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct EpInNormal {
    pub season_id: i64,
    pub section_id: i64,
    pub id: i64,
    pub aid: i64,
    pub cid: i64,
    pub title: String,
    pub attribute: i64,
    pub arc: Arc,
    pub page: PageInNormalEp,
    pub bvid: String,
    pub pages: Vec<PageInNormalEp>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Arc {
    pub aid: i64,
    pub videos: i64,
    pub type_id: i64,
    pub type_name: String,
    pub copyright: i64,
    pub pic: String,
    pub title: String,
    pub pubdate: i64,
    pub ctime: i64,
    pub desc: String,
    pub state: i64,
    pub duration: u64,
    pub rights: RightsInNormalEp,
    pub author: Author,
    pub stat: StatInNormalEp,
    pub dynamic: String,
    pub dimension: Dimension,
    pub is_chargeable_season: bool,
    pub is_blooper: bool,
    pub enable_vt: i64,
    pub vt_display: String,
    pub type_id_v2: i64,
    pub type_name_v2: String,
    pub is_lesson_video: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Author {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct StatInNormalEp {
    pub aid: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub fav: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub dislike: i64,
    pub evaluation: String,
    pub argue_msg: String,
    pub vt: i64,
    pub vv: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct StatInNormalSeason {
    pub season_id: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub fav: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub vt: i64,
    pub vv: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PageInNormalEp {
    pub cid: i64,
    pub page: i64,
    pub from: String,
    pub part: String,
    pub duration: u64,
    pub vid: String,
    pub weblink: String,
    pub dimension: Dimension,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct RightsInNormalEp {
    pub bp: i64,
    pub elec: i64,
    pub download: i64,
    pub movie: i64,
    pub pay: i64,
    pub hd5: i64,
    pub no_reprint: i64,
    pub autoplay: i64,
    pub ugc_pay: i64,
    pub is_cooperation: i64,
    pub ugc_pay_preview: i64,
    pub arc_pay: i64,
    pub free_watch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Staff {
    pub mid: i64,
    pub title: String,
    pub name: String,
    pub face: String,
    pub follower: i64,
    pub label_style: i64,
}
