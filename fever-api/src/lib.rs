use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Serialize, self};

fn join_ids(ids: &[u32], out: &mut String) {
    use std::fmt::Write;

    let mut first = true;
    for &id in ids {
        let sep = if first {""} else {","};
        write!(out, "{}{}", sep, id).unwrap();
        first = false;
    }
}

#[derive(Debug, PartialEq)]
pub enum ApiRequest {
    None,
    Groups,
    Feeds,
    LatestItems,
    ItemsSince(u32),
    ItemsBefore(u32),
    Items(Vec<u32>),
    UnreadItems,
    SavedItems,
    MarkItemRead(u32),
    MarkItemUnread(u32),
    MarkItemSaved(u32),
    MarkItemUnsaved(u32),
    MarkFeedRead(u32, NaiveDateTime),
    MarkGroupRead(u32, NaiveDateTime),
}

impl ApiRequest {
    pub fn parse<'a, I>(mut query_params: I,
                        body_params: &HashMap<String, String>)
            -> Option<ApiRequest>
            where I: Iterator<Item=(&'a str, &'a str)> {
        match query_params.next() {
            Some(("api", "")) => (),
            _ => return None,
        };

        let action = query_params.next().map(|(k, _)| k);
        match action {
            None => {
                let mark = body_params.get("mark").map(|v| &**v);
                let mark_as = body_params.get("as").map(|v| &**v);
                let id = body_params.get("id").and_then(|v| v.parse().ok());
                let before = body_params.get("before")
                    .and_then(|v| v.parse().ok())
                    .map(|t| NaiveDateTime::from_timestamp(t, 0));
                match (mark, mark_as) {
                    (None, None) => Some(ApiRequest::None),
                    (Some("item"), Some("read")) =>
                        id.map(ApiRequest::MarkItemRead),
                    (Some("item"), Some("unread")) =>
                        id.map(ApiRequest::MarkItemUnread),
                    (Some("item"), Some("saved")) =>
                        id.map(ApiRequest::MarkItemSaved),
                    (Some("item"), Some("unsaved")) =>
                        id.map(ApiRequest::MarkItemUnsaved),
                    (Some("feed"), Some("read")) =>
                        id.and_then(|i| before.map(|b| (i, b)))
                          .map(|(i, b)| ApiRequest::MarkFeedRead(i, b)),
                    (Some("group"), Some("read")) =>
                        id.and_then(|i| before.map(|b| (i, b)))
                          .map(|(i, b)| ApiRequest::MarkGroupRead(i, b)),
                    _ => None,
                }
            },
            Some("groups") => Some(ApiRequest::Groups),
            Some("feeds") => Some(ApiRequest::Feeds),
            Some("favicons") => unimplemented!(),
            Some("items") => match query_params.next() {
                Some(("since_id", val)) =>
                    val.parse().ok().map(|v| ApiRequest::ItemsSince(v)),
                Some(("max_id", val)) =>
                    val.parse().ok().map(|v| ApiRequest::ItemsBefore(v)),
                Some(("with_ids", val)) => {
                    let ids: Result<_, _> = val.split(',').map(|v| v.parse()).collect();
                    ids.map(|v| ApiRequest::Items(v)).ok()
                },
                None => Some(ApiRequest::LatestItems),
                _ => None,
            },
            Some("links") => unimplemented!(),
            Some("unread_item_ids") => Some(ApiRequest::UnreadItems),
            Some("saved_item_ids") => Some(ApiRequest::SavedItems),
            _ => None,
        }
    }

    pub fn query(&self) -> String {
        use self::ApiRequest::*;

        match *self {
            None |
            MarkItemRead(_) |
            MarkItemUnread(_) |
            MarkItemSaved(_) |
            MarkItemUnsaved(_) |
            MarkFeedRead(_, _) |
            MarkGroupRead(_, _) => "api".to_owned(),
            Groups => "api&groups".to_owned(),
            Feeds => "api&feeds".to_owned(),
            LatestItems => "api&items".to_owned(),
            ItemsSince(id) =>
                format!("api&items&since_id={}", id),
            ItemsBefore(id) =>
                format!("api&items&max_id={}", id),
            Items(ref ids) => {
                let mut result = "api&items&with_ids=".to_owned();
                join_ids(ids, &mut result);
                result
            }
            UnreadItems => "api&unread_item_ids".to_owned(),
            SavedItems => "api&saved_item_ids".to_owned(),
        }
    }
}

fn serialize_bool_as_number<S>(value: &bool, serializer: &mut S)
        -> Result<(), S::Error>
        where S: serde::Serializer {
    let i = if *value {1} else {0};
    i.serialize(serializer)
}

fn serialize_datetime_as_timestamp<S>(value: &NaiveDateTime, serializer: &mut S)
        -> Result<(), S::Error>
        where S: serde::Serializer {
    let t = value.timestamp();
    t.serialize(serializer)
}

fn serialize_ids_as_comma_string<S>(value: &[u32], serializer: &mut S)
        -> Result<(), S::Error>
        where S: serde::Serializer {
    let mut s = String::new();
    join_ids(value, &mut s);
    s.serialize(serializer)
}

#[derive(Serialize)]
pub struct Group {
    pub id: u32,
    pub title: String,
}

#[derive(Serialize)]
pub struct FeedsGroup {
    pub group_id: u32,
    #[serde(serialize_with = "serialize_ids_as_comma_string")]
    pub feed_ids: Vec<u32>,
}

#[derive(Serialize)]
pub struct Feed {
    pub id: u32,
    // pub favicon_id: u32,
    pub title: String,
    pub url: String,
    // pub site_url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_spark: bool,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub last_updated_on_time: NaiveDateTime,
}

#[derive(Serialize)]
pub struct Item {
    pub id: u32,
    pub feed_id: u32,
    pub title: String,
    // pub author: String,
    pub html: String,
    pub url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_saved: bool,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_read: bool,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub created_on_time: NaiveDateTime,
}

pub enum ApiResponsePayload {
    None,
    Groups {
        groups: Vec<Group>,
        feeds_groups: Vec<FeedsGroup>,
    },
    Feeds {
        feeds: Vec<Feed>,
        feeds_groups: Vec<FeedsGroup>,
    },
    Items {
        items: Vec<Item>,
        total_items: u32,
    },
    UnreadItems {
        unread_item_ids: Vec<u32>,
    },
    SavedItems {
        saved_item_ids: Vec<u32>,
    },
}

impl ApiResponsePayload {
    fn num_fields(&self) -> usize {
        use self::ApiResponsePayload::*;

        match *self {
            None => 0,
            Groups {..} | Feeds {..} | Items {..} => 2,
            UnreadItems {..} | SavedItems {..} => 1,
        }
    }

    fn serialize_fields<S>(&self, state: &mut S::StructState, serializer: &mut S)
            -> Result<(), S::Error>
            where S: serde::Serializer {
        use self::ApiResponsePayload::*;

        match *self {
            Groups { ref groups, ref feeds_groups } => {
                try!(serializer.serialize_struct_elt(state, "groups", groups));
                serializer.serialize_struct_elt(state, "feeds_groups", feeds_groups)
            },
            Feeds { ref feeds, ref feeds_groups } => {
                try!(serializer.serialize_struct_elt(state, "feeds", feeds));
                serializer.serialize_struct_elt(state, "feeds_groups", feeds_groups)
            },
            Items { ref items, total_items } => {
                try!(serializer.serialize_struct_elt(state, "items", items));
                serializer.serialize_struct_elt(state, "total_items", total_items)
            },
            UnreadItems { ref unread_item_ids } => {
                let mut s = String::new();
                join_ids(unread_item_ids, &mut s);
                serializer.serialize_struct_elt(state, "unread_item_ids", s)
            },
            SavedItems { ref saved_item_ids } => {
                let mut s = String::new();
                join_ids(saved_item_ids, &mut s);
                serializer.serialize_struct_elt(state, "saved_item_ids", s)
            },
            None => Ok(())
        }
    }
}

pub struct ApiResponse {
    pub api_version: u32,
    pub auth: bool,
    pub last_refreshed_on_time: Option<NaiveDateTime>,
    pub payload: ApiResponsePayload,
}

impl Serialize for ApiResponse {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let num_fields = 2 + self.payload.num_fields() +
            if self.last_refreshed_on_time.is_some() {1} else {0};
        let mut state = try!(serializer.serialize_struct("ApiResponse", num_fields));

        try!(serializer.serialize_struct_elt(&mut state, "api_version", self.api_version));
        let auth_num = if self.auth {1} else {0};
        try!(serializer.serialize_struct_elt(&mut state, "auth", auth_num));
        if let Some(refresh_time) = self.last_refreshed_on_time {
            let timestamp = refresh_time.timestamp();
            try!(serializer.serialize_struct_elt(&mut state, "last_refreshed_on_time", timestamp));
        }
        try!(self.payload.serialize_fields(&mut state, serializer));

        serializer.serialize_struct_end(state)
    }
}

#[cfg(test)]
mod tests {
    use super::ApiRequest;

    #[test]
    fn test_request_type() {
        assert_eq!(ApiRequest::parse("api"), Some(ApiRequest::None));
        assert_eq!(ApiRequest::parse("api&feeds"), Some(ApiRequest::Feeds));
        assert_eq!(ApiRequest::parse("api&unread_item_ids"),
                   Some(ApiRequest::UnreadItems));
        assert_eq!(ApiRequest::parse("api&items&since_id=0"),
                   Some(ApiRequest::ItemsSince(0)));
        assert_eq!(ApiRequest::parse("api&items&with_ids=0,1,2"),
                   Some(ApiRequest::Items(vec![0, 1, 2])));
    }
}
