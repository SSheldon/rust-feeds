use std::collections::HashMap;

use chrono::NaiveDateTime;

use {Key, join_ids};

#[derive(Debug, PartialEq)]
pub enum RequestType {
    None,
    Groups,
    Feeds,
    Favicons,
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

impl RequestType {
    pub fn parse<'a, I>(mut query_params: I,
                        body_params: &HashMap<String, String>)
            -> Option<RequestType>
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
                    (None, None) => Some(RequestType::None),
                    (Some("item"), Some("read")) =>
                        id.map(RequestType::MarkItemRead),
                    (Some("item"), Some("unread")) =>
                        id.map(RequestType::MarkItemUnread),
                    (Some("item"), Some("saved")) =>
                        id.map(RequestType::MarkItemSaved),
                    (Some("item"), Some("unsaved")) =>
                        id.map(RequestType::MarkItemUnsaved),
                    (Some("feed"), Some("read")) =>
                        id.and_then(|i| before.map(|b| (i, b)))
                          .map(|(i, b)| RequestType::MarkFeedRead(i, b)),
                    (Some("group"), Some("read")) =>
                        id.and_then(|i| before.map(|b| (i, b)))
                          .map(|(i, b)| RequestType::MarkGroupRead(i, b)),
                    _ => None,
                }
            },
            Some("groups") => Some(RequestType::Groups),
            Some("feeds") => Some(RequestType::Feeds),
            Some("favicons") => Some(RequestType::Favicons),
            Some("items") => match query_params.next() {
                Some(("since_id", val)) =>
                    val.parse().ok().map(|v| RequestType::ItemsSince(v)),
                Some(("max_id", val)) =>
                    val.parse().ok().map(|v| RequestType::ItemsBefore(v)),
                Some(("with_ids", val)) => {
                    let ids: Result<_, _> = val.split(',').map(|v| v.parse()).collect();
                    ids.map(|v| RequestType::Items(v)).ok()
                },
                None => Some(RequestType::LatestItems),
                _ => None,
            },
            Some("links") => {
                // TODO: Implement link support
                None
            },
            Some("unread_item_ids") => Some(RequestType::UnreadItems),
            Some("saved_item_ids") => Some(RequestType::SavedItems),
            _ => None,
        }
    }

    pub fn query(&self) -> String {
        use self::RequestType::*;

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
            Favicons => "api&favicons".to_owned(),
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

#[derive(Debug, PartialEq)]
pub struct Request {
    pub req_type: RequestType,
    pub api_key: Key,
}

impl Request {
    pub fn parse<'a, I>(
        query_params: I,
        body_params: &HashMap<String, String>
    ) -> Option<Request>
    where I: Iterator<Item=(&'a str, &'a str)> {
        let api_key = body_params.get("api_key")
            .and_then(|s| s.parse().ok());

        api_key.and_then(|api_key| {
            RequestType::parse(query_params, body_params).map(|req_type| {
                Request { req_type, api_key }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RequestType;

    fn parse_query(query: &str) -> Option<RequestType> {
        None
    }

    #[test]
    fn test_request_type() {
        assert_eq!(parse_query("api"), Some(RequestType::None));
        assert_eq!(parse_query("api&feeds"), Some(RequestType::Feeds));
        assert_eq!(parse_query("api&unread_item_ids"),
                   Some(RequestType::UnreadItems));
        assert_eq!(parse_query("api&items&since_id=0"),
                   Some(RequestType::ItemsSince(0)));
        assert_eq!(parse_query("api&items&with_ids=0,1,2"),
                   Some(RequestType::Items(vec![0, 1, 2])));
    }
}
