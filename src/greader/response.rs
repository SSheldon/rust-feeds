use serde_derive::Serialize;

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UserInfoResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct UnreadCountResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct SubscriptionListResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamContentsResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamItemsIdsResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct StreamItemsContentsResponse {
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Serialize)]
pub struct TagListResponse {
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
