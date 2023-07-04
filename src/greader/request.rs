use chrono::NaiveDateTime;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub enum StreamRanking {
    #[serde(rename = "n")]
    NewestFirst,
    #[serde(rename = "o")]
    OldestFirst,
}

#[derive(Deserialize)]
pub enum StreamState {
    #[serde(rename = "com.google/read")]
    Read,
    #[serde(rename = "com.google/kept-unread")]
    KeptUnread,
    #[serde(rename = "com.google/reading-list")]
    ReadingList,
    #[serde(rename = "com.google/starred")]
    Starred,
}

#[derive(Deserialize)]
pub enum StreamTag {
    Label(Option<String>, String),
    State(Option<String>, StreamState),
}

#[derive(Deserialize)]
pub enum StreamId {
    Feed(String),
    Tag(StreamTag),
}

#[derive(Deserialize)]
pub struct StreamContentsParams {
    #[serde(rename = "r")]
    ranking: StreamRanking,
    #[serde(rename = "n")]
    number: u32,
    #[serde(rename = "c")]
    continuation: String,
    #[serde(rename = "xt")]
    exclude: Option<StreamTag>,
    #[serde(rename = "ot")]
    exclude_older_than: Option<NaiveDateTime>,
    #[serde(rename = "nt")]
    exclude_newer_than: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct StreamItemsIdsParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
    #[serde(rename = "r")]
    ranking: StreamRanking,
    #[serde(rename = "n")]
    number: u32,
    #[serde(rename = "c")]
    continuation: String,
    #[serde(rename = "xt")]
    exclude: Option<StreamTag>,
    #[serde(rename = "ot")]
    exclude_older_than: Option<NaiveDateTime>,
    #[serde(rename = "nt")]
    exclude_newer_than: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct StreamItemsCountParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
}

#[derive(Deserialize)]
pub struct StreamItemsContentsParams {
    #[serde(rename = "i")]
    item_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct EditTagParams {
    #[serde(rename = "i")]
    item_ids: Vec<String>,
    #[serde(rename = "a")]
    add_tags: Vec<StreamTag>,
    #[serde(rename = "r")]
    remove_tags: Vec<StreamTag>,
}

#[derive(Deserialize)]
pub struct MarkAllAsReadParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
    #[serde(rename = "ts")]
    older_than: Option<NaiveDateTime>,
}

pub enum RequestType {
    Token,
    UserInfo,
    UnreadCount,
    SubscriptionList,
    StreamContents(StreamId, StreamContentsParams),
    StreamItemsIds(StreamItemsIdsParams),
    StreamItemsCount(StreamItemsCountParams),
    StreamItemsContents(StreamItemsContentsParams),
    TagList,
    EditTag(EditTagParams),
    MarkAllAsRead(MarkAllAsReadParams),
}
