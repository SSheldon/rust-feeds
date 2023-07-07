use std::fmt;
use std::str::FromStr;

use serde::{self, Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

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
#[derive(Deserialize, Serialize)]
pub enum StreamRanking {
    #[serde(rename = "n")]
    NewestFirst,
    #[serde(rename = "o")]
    OldestFirst,
}

impl StreamRanking {
    fn as_str(self) -> &'static str {
        match self {
            Self::NewestFirst => "n",
            Self::OldestFirst => "o",
        }
    }
}

impl Default for StreamRanking {
    fn default() -> Self {
        Self::NewestFirst
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
            "n" => Ok(Self::NewestFirst),
            "o" => Ok(Self::OldestFirst),
            _ => Err(ParseError { type_name: "StreamRanking", value: s.to_owned() })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
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
            Self::Read => "com.google/read",
            Self::KeptUnread => "com.google/kept-unread",
            Self::ReadingList => "com.google/reading-list",
            Self::Starred => "com.google/starred",
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
            "com.google/read" => Ok(Self::Read),
            "com.google/kept-unread" => Ok(Self::KeptUnread),
            "com.google/reading-list" => Ok(Self::ReadingList),
            "com.google/starred" => Ok(Self::Starred),
            _ => Err(ParseError { type_name: "StreamState", value: s.to_owned() })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamTag {
    Label(Option<String>, String),
    State(Option<String>, StreamState),
}

impl StreamTag {
    fn user(&self) -> Option<&str> {
        match self {
            Self::Label(user, _) => user,
            Self::State(user, _) => user,
        }.as_ref().map(String::as_str)
    }

    fn user_str(&self) -> &str {
        self.user().unwrap_or("-")
    }

    fn type_str(&self) -> &'static str {
        match self {
            Self::Label(_, _) => "label",
            Self::State(_, _) => "state",
        }
    }

    fn value_str(&self) -> &str {
        match self {
            Self::Label(_, label) => label,
            Self::State(_, state) => state.as_str(),
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
            "label" => Self::Label(user, value_str.to_owned()),
            "state" => Self::State(user, StreamState::from_str(value_str)?),
            _ => return Err(make_err()),
        };
        Ok(tag)
    }
}

impl<'de> Deserialize<'de> for StreamTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for StreamTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamId {
    Feed(String),
    Tag(StreamTag),
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Feed(feed) => write!(f, "feed/{}", feed),
            Self::Tag(tag) => tag.fmt(f),
        }
    }
}

impl FromStr for StreamId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(feed) = s.strip_prefix("feed/") {
            Ok(Self::Feed(feed.to_owned()))
        } else if s.starts_with("user/") {
            StreamTag::from_str(s).map(Self::Tag)
        } else {
            Err(ParseError { type_name: "StreamId", value: s.to_owned() })
        }
    }
}

impl<'de> Deserialize<'de> for StreamId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for StreamId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct StreamContentsParams {
    #[serde(rename = "r", default)]
    ranking: StreamRanking,
    #[serde(rename = "n", default)]
    number: u32,
    #[serde(rename = "c", default)]
    continuation: Option<String>,
    #[serde(rename = "xt", default)]
    exclude: Option<StreamTag>,
    #[serde(rename = "ot", default)]
    exclude_older_than: Option<u64>,
    #[serde(rename = "nt", default)]
    exclude_newer_than: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct StreamItemsIdsParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
    #[serde(rename = "r", default)]
    ranking: StreamRanking,
    #[serde(rename = "n", default)]
    number: u32,
    #[serde(rename = "c", default)]
    continuation: Option<String>,
    #[serde(rename = "xt", default)]
    exclude: Option<StreamTag>,
    #[serde(rename = "ot", default)]
    exclude_older_than: Option<u64>,
    #[serde(rename = "nt", default)]
    exclude_newer_than: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct StreamItemsCountParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct StreamItemsContentsParams {
    #[serde(rename = "i")]
    item_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct EditTagParams {
    #[serde(rename = "i")]
    item_ids: Vec<String>,
    #[serde(rename = "a", default)]
    add_tags: Vec<StreamTag>,
    #[serde(rename = "r", default)]
    remove_tags: Vec<StreamTag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
pub struct MarkAllAsReadParams {
    #[serde(rename = "s")]
    stream_id: StreamId,
    #[serde(rename = "ts", default)]
    older_than: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl RequestType {
    pub fn parse(path: &str, params: &str) -> Option<RequestType> {
        Some(match path {
            "token" => Self::Token,
            "user-info" => Self::UserInfo,
            "unread-count" => Self::UnreadCount,
            "subscription/list" => Self::SubscriptionList,
            // stream/contents
            "stream/items/ids" => Self::StreamItemsIds(serde_html_form::from_str(params).ok()?),
            "stream/items/count" => Self::StreamItemsCount(serde_html_form::from_str(params).ok()?),
            "stream/items/contents" => Self::StreamItemsContents(serde_html_form::from_str(params).ok()?),
            "tag/list" => Self::TagList,
            "edit-tag" => Self::EditTag(serde_html_form::from_str(params).ok()?),
            "mark-all-as-read" => Self::MarkAllAsRead(serde_html_form::from_str(params).ok()?),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_parse() {
        assert_eq!(
            StreamTag::from_str("user/-/state/com.google/starred"),
            Ok(StreamTag::State(None, StreamState::Starred)),
        );
        assert_eq!(
            StreamTag::from_str("user/1/label/Blogs"),
            Ok(StreamTag::Label(Some("1".to_owned()), "Blogs".to_owned())),
        );
        assert!(StreamTag::from_str("user/-/tag/foo").is_err());
        assert!(StreamTag::from_str("user/label/Blogs").is_err());
        assert!(StreamTag::from_str("label/Blogs").is_err());
    }

    #[test]
    fn test_id_parse() {
        assert_eq!(
            StreamId::from_str("feed/foo"),
            Ok(StreamId::Feed("foo".to_owned())),
        );
        assert_eq!(
            StreamId::from_str("user/-/state/com.google/starred"),
            Ok(StreamId::Tag(StreamTag::State(None, StreamState::Starred))),
        );
        assert!(StreamTag::from_str("user/-/feed/foo").is_err());
    }

    #[test]
    fn test_deserialize_count_params() {
        assert_eq!(
            serde_html_form::from_str("s=user/-/state/com.google/reading-list"),
            Ok(StreamItemsCountParams {
                stream_id: StreamId::Tag(StreamTag::State(None, StreamState::ReadingList)),
            }),
        );
    }

    #[test]
    fn test_deserialize_item_contents_params() {
        assert_eq!(
            serde_html_form::from_str("i=1&i=2"),
            Ok(StreamItemsContentsParams {
                item_ids: vec!["1".to_owned(), "2".to_owned()],
            }),
        );
    }

    #[test]
    fn test_deserialize_edit_tags_params() {
        assert_eq!(
            serde_html_form::from_str("i=1&r=user/-/state/com.google/starred"),
            Ok(EditTagParams {
                item_ids: vec!["1".to_owned()],
                add_tags: vec![],
                remove_tags: vec![StreamTag::State(None, StreamState::Starred)],
            }),
        );
    }
}
