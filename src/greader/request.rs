use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde_derive::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    type_name: &'static str,
    value: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid {}: {:?}", self.type_name, self.value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub enum StreamRanking {
    #[serde(rename = "n")]
    NewestFirst,
    #[serde(rename = "o")]
    OldestFirst,
}

impl StreamRanking {
    fn as_str(self) -> &'static str {
        match self {
            StreamRanking::NewestFirst => "n",
            StreamRanking::OldestFirst => "o",
        }
    }
}

impl fmt::Display for StreamRanking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StreamRanking {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" => Ok(StreamRanking::NewestFirst),
            "o" => Ok(StreamRanking::OldestFirst),
            _ => Err(ParseError { type_name: "StreamRanking", value: s.to_owned() })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl StreamState {
    fn as_str(self) -> &'static str {
        match self {
            StreamState::Read => "com.google/read",
            StreamState::KeptUnread => "com.google/kept-unread",
            StreamState::ReadingList => "com.google/reading-list",
            StreamState::Starred => "com.google/starred",
        }
    }
}

impl fmt::Display for StreamState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StreamState {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "com.google/read" => Ok(StreamState::Read),
            "com.google/kept-unread" => Ok(StreamState::KeptUnread),
            "com.google/reading-list" => Ok(StreamState::ReadingList),
            "com.google/starred" => Ok(StreamState::Starred),
            _ => Err(ParseError { type_name: "StreamState", value: s.to_owned() })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub enum StreamTag {
    Label(Option<String>, String),
    State(Option<String>, StreamState),
}

impl StreamTag {
    fn user(&self) -> Option<&str> {
        match self {
            StreamTag::Label(user, _) => user,
            StreamTag::State(user, _) => user,
        }.as_ref().map(String::as_str)
    }

    fn user_str(&self) -> &str {
        self.user().unwrap_or("-")
    }

    fn type_str(&self) -> &'static str {
        match self {
            StreamTag::Label(_, _) => "label",
            StreamTag::State(_, _) => "state",
        }
    }

    fn value_str(&self) -> &str {
        match self {
            StreamTag::Label(_, label) => label,
            StreamTag::State(_, state) => state.as_str(),
        }
    }
}

impl fmt::Display for StreamTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "user/{}/{}/{}", self.user_str(), self.type_str(), self.value_str())
    }
}

impl FromStr for StreamTag {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let make_err = || ParseError { type_name: "StreamTag", value: s.to_owned() };

        let (user_str, remaining) = s.strip_prefix("user/")
            .and_then(|s| s.split_once('/'))
            .ok_or_else(make_err)?;
        let user = if user_str == "-" { None } else { Some(user_str.to_owned()) };

        let (type_str, value_str) = remaining.split_once('/').ok_or_else(make_err)?;
        let tag = match type_str {
            "label" => StreamTag::Label(user, value_str.to_owned()),
            "state" => StreamTag::State(user, StreamState::from_str(value_str)?),
            _ => return Err(make_err()),
        };
        Ok(tag)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub enum StreamId {
    Feed(String),
    Tag(StreamTag),
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamId::Feed(feed) => write!(f, "feed/{}", feed),
            StreamId::Tag(tag) => tag.fmt(f),
        }
    }
}

impl FromStr for StreamId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(feed) = s.strip_prefix("feed/") {
            Ok(StreamId::Feed(feed.to_owned()))
        } else if s.starts_with("user/") {
            StreamTag::from_str(s).map(StreamId::Tag)
        } else {
            Err(ParseError { type_name: "StreamId", value: s.to_owned() })
        }
    }
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
