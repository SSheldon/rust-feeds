use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

use super::request::{ItemId, StreamId, StreamTag};
use super::timestamp::{self, Convert};

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user_id: String,
    pub user_name: String,
    pub user_profile_id: String,
    pub user_email: String,
    pub is_blogger_user: bool,
    #[serde(rename = "signupTimeSec", with = "timestamp::Sec")]
    pub signup_time: NaiveDateTime,
    pub public_user_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct UnreadCountResponse {
    pub max: u32,
    #[serde(rename = "unreadcounts")]
    pub unread_counts: Vec<UnreadCount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct UnreadCount {
    pub count: u32,
    pub id: StreamId,
    #[serde(rename = "newestItemTimestampUsec", with = "timestamp::USec")]
    pub newest_item_time: NaiveDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct Subscription {
    pub title: String,
    #[serde(rename = "firstitemmsec", with = "timestamp::MSec")]
    pub first_item_time: NaiveDateTime,
    #[serde(rename = "htmlUrl")]
    pub html_url: String,
    #[serde(rename = "sortid")]
    pub sort_id: String,
    pub id: StreamId,
    pub categories: Vec<SubscriptionCategory>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct SubscriptionCategory {
    pub id: StreamTag,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct StreamContentsResponse {
    pub direction: String,
    pub author: String,
    pub title: String,
    #[serde(with = "timestamp::Sec")]
    pub updated: NaiveDateTime,
    pub continuation: String,
    pub id: StreamId,
    #[serde(rename = "self")]
    pub self_links: Vec<Link>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct Link {
    pub href: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct Item {
    pub id: ItemId,
    pub origin: ItemOrigin,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<StreamTag>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternate: Vec<Link>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub title: String,
    pub summary: ItemSummary,
    #[serde(rename = "timestampUsec", with = "timestamp::USec")]
    pub timestamp: NaiveDateTime,
    #[serde(with = "timestamp::Sec")]
    pub published: NaiveDateTime,
    #[serde(with = "timestamp::OptSec", default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<NaiveDateTime>,
    #[serde(rename = "crawlTimeMsec", with = "timestamp::OptMSec", default, skip_serializing_if = "Option::is_none")]
    pub crawl_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOrigin {
    pub stream_id: StreamId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct ItemSummary {
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamItemsIdsResponse {
    pub item_refs: Vec<ItemRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRef {
    pub id: ItemId,
    #[serde(rename = "timestampUsec", with = "timestamp::USec")]
    pub timestamp: NaiveDateTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub direct_stream_ids: Vec<StreamId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct StreamItemsContentsResponse {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct TagListResponse {
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub id: StreamTag,
    #[serde(rename = "sortid", default, skip_serializing_if = "Option::is_none")]
    pub sort_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Plain(String),
    UserInfo(UserInfoResponse),
    UnreadCount(UnreadCountResponse),
    SubscriptionList(SubscriptionListResponse),
    StreamContents(StreamContentsResponse),
    StreamItemsIds(StreamItemsIdsResponse),
    StreamItemsContents(StreamItemsContentsResponse),
    TagList(TagListResponse),
}

impl From<String> for Response {
    fn from(value: String) -> Self { Self::Plain(value) }
}

impl From<UserInfoResponse> for Response {
    fn from(value: UserInfoResponse) -> Self { Self::UserInfo(value) }
}

impl From<UnreadCountResponse> for Response {
    fn from(value: UnreadCountResponse) -> Self { Self::UnreadCount(value) }
}

impl From<SubscriptionListResponse> for Response {
    fn from(value: SubscriptionListResponse) -> Self { Self::SubscriptionList(value) }
}

impl From<StreamContentsResponse> for Response {
    fn from(value: StreamContentsResponse) -> Self { Self::StreamContents(value) }
}

impl From<StreamItemsIdsResponse> for Response {
    fn from(value: StreamItemsIdsResponse) -> Self { Self::StreamItemsIds(value) }
}

impl From<StreamItemsContentsResponse> for Response {
    fn from(value: StreamItemsContentsResponse) -> Self { Self::StreamItemsContents(value) }
}

impl From<TagListResponse> for Response {
    fn from(value: TagListResponse) -> Self { Self::TagList(value) }
}
