use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{self, Deserialize};
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

impl Default for StreamRanking {
    fn default() -> Self {
        StreamRanking::NewestFirst
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

impl<'de> Deserialize<'de> for StreamTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
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

impl<'de> Deserialize<'de> for StreamId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
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
    exclude_older_than: Option<NaiveDateTime>,
    #[serde(rename = "nt", default)]
    exclude_newer_than: Option<NaiveDateTime>,
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
    exclude_older_than: Option<NaiveDateTime>,
    #[serde(rename = "nt", default)]
    exclude_newer_than: Option<NaiveDateTime>,
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
    older_than: Option<NaiveDateTime>,
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
            "token" => RequestType::Token,
            "user-info" => RequestType::UserInfo,
            "unread-count" => RequestType::UnreadCount,
            "subscription/list" => RequestType::SubscriptionList,
            // stream/contents
            "stream/items/ids" => RequestType::StreamItemsIds(serde_html_form::from_str(params).ok()?),
            "stream/items/count" => RequestType::StreamItemsCount(serde_html_form::from_str(params).ok()?),
            "stream/items/contents" => RequestType::StreamItemsContents(serde_html_form::from_str(params).ok()?),
            "tag/list" => RequestType::TagList,
            "edit-tag" => RequestType::EditTag(serde_html_form::from_str(params).ok()?),
            "mark-all-as-read" => RequestType::MarkAllAsRead(serde_html_form::from_str(params).ok()?),
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
