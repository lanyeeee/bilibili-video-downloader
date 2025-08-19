use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BangumiInfo {
    pub activity: Activity,
    pub actors: String,
    pub alias: String,
    pub areas: Vec<AreaInBangumi>,
    pub bkg_cover: String,
    pub cover: String,
    pub delivery_fragment_video: bool,
    pub enable_vt: bool,
    pub episodes: Vec<EpInBangumi>,
    pub evaluate: String,
    pub hide_ep_vv_vt_dm: i64,
    pub icon_font: IconFont,
    pub jp_title: String,
    pub link: String,
    pub media_id: i64,
    pub mode: i64,
    pub new_ep: NewEp,
    pub payment: Option<PaymentInBangumi>,
    pub play_strategy: Option<PlayStrategy>,
    pub positive: Positive,
    pub publish: PublishInBangumi,
    pub rating: Option<RatingInBangumi>,
    pub record: String,
    pub rights: RightsInBangumi,
    pub season_id: i64,
    pub season_title: String,
    pub seasons: Vec<Season>,
    pub section: Option<Vec<SectionInBangumi>>,
    pub series: SeriesInBangumi,
    pub share_copy: String,
    pub share_sub_title: String,
    pub share_url: String,
    pub show: Show,
    pub show_season_type: i64,
    pub square_cover: String,
    pub staff: String,
    pub stat: StatInBangumi,
    pub status: i64,
    pub styles: Vec<String>,
    pub subtitle: String,
    pub title: String,
    pub total: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub up_info: Option<UpInfoInBangumi>,
    pub user_status: UserStatusInBangumi,
}

impl BangumiInfo {
    #[allow(clippy::cast_possible_wrap)]
    pub fn get_episode_with_order(&self, ep_id: i64) -> anyhow::Result<(&EpInBangumi, i64)> {
        let episode_with_order = self
            .episodes
            .iter()
            .enumerate()
            .map(|(i, ep)| (ep, i as i64 + 1))
            .find(|(ep, _)| ep.id == ep_id);

        let episode_with_order = if let Some(episode_with_order) = episode_with_order {
            // 如果在正片中找到了对应的ep_id
            episode_with_order
        } else {
            // 如果在正片中没有找到对应的ep_id，则在section中查找
            let Some(sections) = &self.section else {
                return Err(anyhow!("找不到对应的ep_id为`{ep_id}`的番剧"));
            };
            let section_index = sections
                .iter()
                .position(|s| s.episodes.iter().any(|e| e.id == ep_id))
                .context(format!("找不到含有ep_id为`{ep_id}`的ep的section"))?;
            sections[section_index]
                .episodes
                .iter()
                .enumerate()
                .map(|(i, e)| (e, i as i64 + 1))
                .find(|(e, _)| e.id == ep_id)
                .context(format!("在section中找不到ep_id为`{ep_id}`的ep"))?
        };

        Ok(episode_with_order)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Activity {
    pub head_bg_url: String,
    pub id: i64,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AreaInBangumi {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct EpInBangumi {
    pub aid: i64,
    pub badge: String,
    pub badge_info: BadgeInfoInBangumi,
    pub badge_type: Option<i64>,
    pub bvid: Option<String>,
    pub cid: i64,
    pub cover: String,
    pub dimension: Option<DimensionInBangumi>,
    pub duration: Option<u64>,
    pub enable_vt: bool,
    pub ep_id: i64,
    pub from: Option<String>,
    pub id: i64,
    pub is_view_hide: bool,
    pub link: String,
    pub link_type: Option<String>,
    pub long_title: Option<String>,
    pub pub_time: i64,
    pub pv: i64,
    pub release_date: Option<String>,
    pub rights: Option<RightsInBangumiEp>,
    pub section_type: i64,
    pub share_copy: Option<String>,
    pub share_url: Option<String>,
    pub short_link: Option<String>,
    #[serde(rename = "showDrmLoginDialog")]
    pub show_drm_login_dialog: bool,
    pub show_title: Option<String>,
    pub skip: Option<Skip>,
    pub status: i64,
    pub subtitle: Option<String>,
    pub title: String,
    pub vid: Option<String>,
    pub icon_font: Option<IconFont>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BadgeInfoInBangumi {
    pub bg_color: String,
    pub bg_color_night: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DimensionInBangumi {
    pub height: i64,
    pub rotate: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RightsInBangumiEp {
    pub allow_dm: i64,
    pub allow_download: i64,
    pub area_limit: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Skip {
    pub ed: Ed,
    pub op: Op,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Ed {
    pub end: i64,
    pub start: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Op {
    pub end: i64,
    pub start: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct IconFont {
    pub name: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct NewEp {
    pub desc: String,
    pub id: i64,
    pub is_new: i64,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PaymentInBangumi {
    pub discount: i64,
    pub pay_type: PayType,
    pub price: String,
    pub promotion: String,
    pub tip: String,
    pub view_start_time: i64,
    pub vip_discount: i64,
    pub vip_first_promotion: String,
    pub vip_price: String,
    pub vip_promotion: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PayType {
    pub allow_discount: i64,
    pub allow_pack: i64,
    pub allow_ticket: i64,
    pub allow_time_limit: i64,
    pub allow_vip_discount: i64,
    pub forbid_bb: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PlayStrategy {
    pub strategies: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Positive {
    pub id: i64,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PublishInBangumi {
    pub is_finish: i64,
    pub is_started: i64,
    pub pub_time: String,
    pub pub_time_show: String,
    pub unknow_pub_date: i64,
    pub weekday: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RatingInBangumi {
    pub count: i64,
    pub score: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RightsInBangumi {
    pub allow_bp: i64,
    pub allow_bp_rank: i64,
    pub allow_download: i64,
    pub allow_review: i64,
    pub area_limit: i64,
    pub ban_area_show: i64,
    pub can_watch: i64,
    pub copyright: String,
    pub forbid_pre: i64,
    pub freya_white: i64,
    pub is_cover_show: i64,
    pub is_preview: i64,
    pub only_vip_download: i64,
    pub resource: String,
    pub watch_platform: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct Season {
    pub badge: String,
    pub badge_info: BadgeInfoInBangumi,
    pub badge_type: i64,
    pub cover: String,
    pub enable_vt: bool,
    pub horizontal_cover_1610: String,
    pub horizontal_cover_169: String,
    pub icon_font: IconFont,
    pub media_id: i64,
    pub new_ep: NewEpInSeason,
    pub season_id: i64,
    pub season_title: String,
    pub season_type: i64,
    pub stat: StatInSeason,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct NewEpInSeason {
    pub cover: String,
    pub id: i64,
    pub index_show: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StatInSeason {
    pub favorites: i64,
    pub series_follow: i64,
    pub views: i64,
    pub vt: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct SeriesInBangumi {
    pub display_type: i64,
    pub series_id: i64,
    pub series_title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Show {
    pub wide_screen: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StatInBangumi {
    pub coins: i64,
    pub danmakus: i64,
    pub favorite: i64,
    pub favorites: i64,
    pub follow_text: String,
    pub likes: i64,
    pub reply: i64,
    pub share: i64,
    pub views: i64,
    pub vt: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UpInfoInBangumi {
    pub avatar: String,
    pub mid: i64,
    pub uname: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserStatusInBangumi {
    pub area_limit: i64,
    pub ban_area_show: i64,
    pub follow: i64,
    pub follow_status: i64,
    pub login: i64,
    pub pay: i64,
    pub pay_pack_paid: i64,
    pub sponsor: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SectionInBangumi {
    pub attr: i64,
    pub episodes: Vec<EpInBangumi>,
    pub id: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub type2: i64,
}
