use serde_derive::Serialize;

use super::request::{StreamId, StreamTag};

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UserInfoResponse {
    user_id: String,
    user_name: String,
    user_profile_id: String,
    user_email: String,
    is_blogger_user: bool,
    signup_time_sec: u64,
    public_user_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UnreadCountResponse {
    max: u32,
    unreadcounts: Vec<UnreadCount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UnreadCount {
    count: u32,
    id: StreamId,
    newest_item_timestamp_usec: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct SubscriptionListResponse {
    subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Subscription {
    title: String,
    first_item_msec: String,
    html_url: String,
    sort_id: String,
    id: StreamId,
    categories: Vec<SubscriptionCategories>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct SubscriptionCategories {
    id: StreamTag,
    label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamContentsResponse {
    direction: String,
    author: String,
    title: String,
    updated: u64,
    continuation: String,
    id: StreamId,
    self_links: Vec<Link>,
    items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Link {
    href: String,
    link_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Item {
    updated: u64,
    id: String,
    categories: Vec<StreamTag>,
    author: String,
    alternate: Vec<Link>,
    timestamp_usec: String,
    content: ItemContent,
    crawl_time_msec: String,
    published: u64,
    title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct ItemContent {
    direction: String,
    content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamItemsIdsResponse {
    item_refs: Vec<ItemRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct ItemRef {
    id: String,
    timestamp_usec: String,
    direct_stream_ids: Vec<StreamId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamItemsContentsResponse {
    items: Vec<Item>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct TagListResponse {
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct Tag {
    id: StreamTag,
    sort_id: String,
}

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
