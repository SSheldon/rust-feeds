use std::collections::BTreeMap;

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
    pub fn parse(query: &str) -> Option<ApiRequest> {
        let components: Vec<_> = query.split('&').map(|c| {
            let mut split = c.splitn(2, '=');
            (split.next().unwrap(), split.next())
        }).collect();

        let components = match components.split_first() {
            Some((&("api", None), components)) => components,
            _ => return None,
        };
        let (action, components) = match components.split_first() {
            Some((&action, components)) => (action, components),
            None => return Some(ApiRequest::None),
        };

        match action {
            ("groups", None) => Some(ApiRequest::Groups),
            ("feeds", None) => Some(ApiRequest::Feeds),
            ("favicons", None) => unimplemented!(),
            ("items", None) => match components.first() {
                Some(&("since_id", val)) => val.and_then(|v| v.parse().ok())
                                               .map(|v| ApiRequest::ItemsSince(v)),
                Some(&("max_id", val)) => val.and_then(|v| v.parse().ok())
                                             .map(|v| ApiRequest::ItemsBefore(v)),
                Some(&("with_ids", val)) => val.and_then(|v| {
                    let ids: Result<_, _> = v.split(',').map(|v| v.parse()).collect();
                    ids.map(|v| ApiRequest::Items(v)).ok()
                }),
                None => Some(ApiRequest::LatestItems),
                _ => None,
            },
            ("links", None) => unimplemented!(),
            ("unread_item_ids", None) => Some(ApiRequest::UnreadItems),
            ("saved_item_ids", None) => Some(ApiRequest::SavedItems),
            ("mark", Some("item")) => {
                unimplemented!();
            }
            ("mark", Some("feed")) => {
                unimplemented!();
            }
            ("mark", Some("group")) => {
                unimplemented!();
            }
            _ => None,
        }
    }

    pub fn query(&self) -> String {
        use std::fmt::Write;
        use self::ApiRequest::*;

        match *self {
            None => "api".to_owned(),
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
            MarkItemRead(id) =>
                format!("api&mark=item&as=read&id={}", id),
            MarkItemSaved(id) =>
                format!("api&mark=item&as=saved&id={}", id),
            MarkItemUnsaved(id) =>
                format!("api&mark=item&as=unsaved&id={}", id),
            MarkFeedRead(id, before) =>
                format!("api&mark=feed&as=read&id={}&before={}", id, before),
            MarkGroupRead(id, before) =>
                format!("api&mark=group&as=read&id={}&before={}", id, before),
        }
    }
}

pub struct Group {
    pub id: u32,
    pub title: String,
    pub feed_ids: Vec<u32>,
}

#[derive(Default)]
pub struct Feed {
    pub id: u32,
    pub favicon_id: u32,
    pub title: String,
    pub url: String,
    pub site_url: String,
    pub is_spark: bool,
    pub last_updated_on_time: i32,
}

impl Feed {
    pub fn into_json(self) -> Value {
        let mut obj = BTreeMap::new();
        obj.insert("id".to_owned(), Value::U64(self.id as u64));
        // obj.insert("favicon_id".to_owned(), Value::U64(self.favicon_id as u64));
        obj.insert("title".to_owned(), Value::String(self.title));
        obj.insert("url".to_owned(), Value::String(self.url));
        // obj.insert("site_url".to_owned(), Value::String(self.site_url));
        // obj.insert("is_spark".to_owned(), Value::U64(if self.is_spark {1} else {0}));
        obj.insert("last_updated_on_time".to_owned(), Value::I64(self.last_updated_on_time as i64));
        Value::Object(obj)
    }
}

#[derive(Default)]
pub struct Item {
    pub id: u32,
    pub feed_id: u32,
    pub title: String,
    pub author: String,
    pub html: String,
    pub url: String,
    pub is_saved: bool,
    pub is_read: bool,
    pub created_on_time: i32,
}

impl Item {
    pub fn into_json(self) -> Value {
        let mut obj = BTreeMap::new();
        obj.insert("id".to_owned(), Value::U64(self.id as u64));
        obj.insert("feed_id".to_owned(), Value::U64(self.feed_id as u64));
        obj.insert("title".to_owned(), Value::String(self.title));
        // obj.insert("author".to_owned(), Value::String(self.author));
        obj.insert("html".to_owned(), Value::String(self.html));
        obj.insert("url".to_owned(), Value::String(self.url));
        obj.insert("is_saved".to_owned(), Value::U64(if self.is_saved {1} else {0}));
        obj.insert("is_read".to_owned(), Value::U64(if self.is_read {1} else {0}));
        obj.insert("created_on_time".to_owned(), Value::I64(self.created_on_time as i64));
        Value::Object(obj)
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
