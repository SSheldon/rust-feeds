use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

use super::request::{ItemId, StreamId, StreamTag};
use super::timestamp::{self, Convert};

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user_id: String,
    pub user_profile_id: String,
    pub user_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct UnreadCountResponse {
    #[serde(rename = "unreadcounts")]
    pub unread_counts: Vec<UnreadCount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct UnreadCount {
    pub count: u32,
    pub id: StreamId,
    #[serde(rename = "newestItemTimestampUsec", with = "timestamp::OptUSec", default, skip_serializing_if = "Option::is_none")]
    pub newest_item_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub struct Subscription {
    pub id: StreamId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "htmlUrl", default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(rename = "sortid", default, skip_serializing_if = "Option::is_none")]
    pub sort_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    pub id: StreamId,
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternate: Vec<Link>,
    #[serde(with = "timestamp::OptSec", default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<NaiveDateTime>,
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
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
