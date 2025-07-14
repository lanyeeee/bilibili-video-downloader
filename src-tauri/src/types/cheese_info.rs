use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
pub struct CheeseInfo {
    pub abtest_info: AbtestInfo,
    pub be_subscription: bool,
    pub brief: Brief,
    pub consulting: Consulting,
    pub cooperation: Cooperation,
    pub course_content: String,
    pub cover: String,
    pub ep_count: i64,
    pub episode_page: EpPage,
    pub episode_sort: i64,
    pub episode_tag: EpTag,
    pub episodes: Vec<EpInCheese>,
    pub expiry_day: i64,
    pub expiry_info_content: String,
    pub faq: Faq,
    pub faq1: Faq1,
    pub is_enable_cash: bool,
    pub is_series: bool,
    pub live_ep_count: i64,
    pub opened_ep_count: i64,
    pub paid_jump: PaidJump,
    pub paid_view: bool,
    pub payment: Payment,
    pub previewed_purchase_note: PreviewedPurchaseNote,
    pub purchase_format_note: PurchaseFormatNote,
    pub purchase_note: PurchaseNote,
    pub purchase_protocol: PurchaseProtocol,
    pub recommend_seasons: Vec<RecommendSeason>,
    pub release_bottom_info: String,
    pub release_info: String,
    pub release_info2: String,
    pub release_status: String,
    pub season_id: i64,
    pub season_tag: i64,
    pub share_url: String,
    pub short_link: String,
    pub show_watermark: bool,
    pub stat: StatInCheese,
    pub status: i64,
    pub stop_sell: bool,
    pub subscription_update_count_cycle_text: String,
    pub subtitle: String,
    pub title: String,
    pub up_info: UpInfoInCheese,
    pub update_status: i64,
    pub user_status: UserStatusInCheese,
    pub watermark_interval: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AbtestInfo {
    pub style_abtest: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Brief {
    pub content: String,
    pub img: Vec<Img>,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Img {
    pub aspect_ratio: f64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Consulting {
    pub consulting_flag: bool,
    pub consulting_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Cooperation {
    pub link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EpPage {
    pub next: bool,
    pub num: i64,
    pub size: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_field_names)]
pub struct EpTag {
    pub part_preview_tag: String,
    pub pay_tag: String,
    pub preview_tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[allow(clippy::struct_excessive_bools)]
#[allow(clippy::struct_field_names)]
pub struct EpInCheese {
    pub aid: i64,
    pub catalogue_index: i64,
    pub cid: i64,
    pub cover: String,
    pub duration: u64,
    pub ep_status: i64,
    pub episode_can_view: bool,
    pub from: String,
    pub id: i64,
    pub index: i64,
    pub label: Option<String>,
    pub page: i64,
    pub play: i64,
    pub play_way: i64,
    pub playable: bool,
    pub release_date: i64,
    pub show_vt: bool,
    pub status: i64,
    pub subtitle: String,
    pub title: String,
    pub watched: bool,
    #[serde(rename = "watchedHistory")]
    pub watched_history: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Faq {
    pub content: String,
    pub link: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Faq1 {
    pub items: Vec<Faq1Item>,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Faq1Item {
    pub answer: String,
    pub question: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PaidJump {
    pub jump_url_for_app: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Payment {
    pub bp_enough: i64,
    pub desc: String,
    pub my_bp: i64,
    pub pay_shade: String,
    pub price: f64,
    pub price_format: String,
    pub price_unit: String,
    pub refresh_text: String,
    pub select_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PreviewedPurchaseNote {
    pub long_watch_text: String,
    pub pay_text: String,
    pub price_format: String,
    pub watch_text: String,
    pub watching_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PurchaseFormatNote {
    pub content_list: Vec<ContentList>,
    pub link: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ContentList {
    pub bold: bool,
    pub content: String,
    pub number: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PurchaseNote {
    pub content: String,
    pub link: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PurchaseProtocol {
    pub link: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RecommendSeason {
    pub cover: String,
    pub ep_count: String,
    pub id: i64,
    pub season_url: String,
    pub subtitle: String,
    pub title: String,
    pub view: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StatInCheese {
    pub play: i64,
    pub play_desc: String,
    pub show_vt: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UpInfoInCheese {
    pub avatar: String,
    pub brief: String,
    pub follower: i64,
    pub is_follow: i64,
    pub is_living: bool,
    pub link: String,
    pub mid: i64,
    pub pendant: PendantInCheese,
    pub season_count: i64,
    pub uname: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PendantInCheese {
    pub image: String,
    pub name: String,
    pub pid: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UserStatusInCheese {
    pub bp: i64,
    pub expire_at: i64,
    pub favored: i64,
    pub favored_count: i64,
    pub is_expired: bool,
    pub is_first_paid: bool,
    pub payed: i64,
    pub user_expiry_content: String,
}
