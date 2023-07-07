use serde_derive::Serialize;

use super::request::{ItemId, StreamId, StreamTag};

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub user_id: String,
    pub user_name: String,
    pub user_profile_id: String,
    pub user_email: String,
    pub is_blogger_user: bool,
    pub signup_time_sec: u64,
    pub public_user_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UnreadCountResponse {
    pub max: u32,
    #[serde(rename = "unreadcounts")]
    pub unread_counts: Vec<UnreadCount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadCount {
    pub count: u32,
    pub id: StreamId,
    pub newest_item_timestamp_usec: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Subscription {
    pub title: String,
    #[serde(rename = "firstitemmsec")]
    pub first_item_msec: String,
    #[serde(rename = "htmlUrl")]
    pub html_url: String,
    #[serde(rename = "sortid")]
    pub sort_id: String,
    pub id: StreamId,
    pub categories: Vec<SubscriptionCategories>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct SubscriptionCategories {
    pub id: StreamTag,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamContentsResponse {
    pub direction: String,
    pub author: String,
    pub title: String,
    pub updated: u64,
    pub continuation: String,
    pub id: StreamId,
    #[serde(rename = "self")]
    pub self_links: Vec<Link>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Link {
    pub href: String,
    pub link_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub updated: u64,
    pub id: ItemId,
    pub categories: Vec<StreamTag>,
    pub author: String,
    pub alternate: Vec<Link>,
    pub timestamp_usec: String,
    pub content: ItemContent,
    pub crawl_time_msec: String,
    pub published: u64,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct ItemContent {
    pub direction: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamItemsIdsResponse {
    pub item_refs: Vec<ItemRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRef {
    pub id: ItemId,
    pub timestamp_usec: String,
    pub direct_stream_ids: Vec<StreamId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamItemsContentsResponse {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct TagListResponse {
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Tag {
    pub id: StreamTag,
    #[serde(rename = "sortid")]
    pub sort_id: String,
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
