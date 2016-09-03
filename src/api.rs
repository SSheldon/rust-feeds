use std::collections::BTreeMap;

use serde::{Serialize, self};
use serde_json::Value;

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
    MarkItemSaved(u32),
    MarkItemUnsaved(u32),
    MarkFeedRead(u32, i32),
    MarkGroupRead(u32, i32),
}

impl ApiRequest {
    pub fn parse<'a, I>(mut query_params: I) -> Option<ApiRequest>
            where I: Iterator<Item=(&'a str, &'a str)> {
        match query_params.next() {
            Some(("api", "")) => (),
            _ => return None,
        };

        let action = query_params.next().map(|(k, _)| k);
        match action {
            // TODO: there may still be post params with no action
            None => Some(ApiRequest::None),
            Some("groups") => Some(ApiRequest::Groups),
            Some("feeds") => Some(ApiRequest::Feeds),
            Some("favicons") => unimplemented!(),
            Some("items") => match query_params.next() {
                Some(("since_id", val)) => val.parse().ok()
                                              .map(|v| ApiRequest::ItemsSince(v)),
                Some(("max_id", val)) => val.parse().ok()
                                            .map(|v| ApiRequest::ItemsBefore(v)),
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
        use std::fmt::Write;
        use self::ApiRequest::*;

        match *self {
            None  |
            MarkItemRead(_) |
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
                let mut first = true;
                for &id in ids {
                    let sep = if first {""} else {","};
                    write!(&mut result, "{}{}", sep, id).unwrap();
                    first = false;
                }
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

pub struct Group {
    pub id: u32,
    pub title: String,
    pub feed_ids: Vec<u32>,
}

#[derive(Default, Serialize)]
pub struct Feed {
    pub id: u32,
    // pub favicon_id: u32,
    pub title: String,
    pub url: String,
    // pub site_url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_spark: bool,
    pub last_updated_on_time: i32,
}

#[derive(Default, Serialize)]
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
    pub created_on_time: i32,
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
