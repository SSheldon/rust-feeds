use chrono::NaiveDateTime;
use reqwest;

use feed_stream::{Entry, FeedParser};
use fever_api::{ApiRequest, ApiResponse, ApiResponsePayload, Feed, Item};

fn item_from_entry(entry: Entry, id: u32, feed: &Feed) -> Item {
    Item {
        id: id,
        feed_id: feed.id,
        title: entry.title().to_owned(),
        url: entry.link().unwrap().to_owned(),
        html: entry.content().unwrap().into_owned(),
        is_saved: false,
        is_read: false,
        created_on_time: entry.published().unwrap().naive_utc(),
    }
}

fn fetch_items(feed: &Feed) -> ApiResponsePayload {
    let response = reqwest::get("https://xkcd.com/atom.xml").unwrap();
    let parser = FeedParser::new(response);
    let items: Vec<_> = parser
        .map(|entry| entry.unwrap())
        .enumerate()
        .map(|(i, entry)| item_from_entry(entry, 100 - (i as u32), feed))
        .collect();
    let total_items = items.len() as u32;

    ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    }
}

pub fn handle_api_request(req_type: &ApiRequest) -> ApiResponse {
    let feed = Feed {
        id: 1,
        title: "xkcd.com".to_owned(),
        url: "https://xkcd.com/".to_owned(),
        is_spark: false,
        last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
    };

    let payload = match *req_type {
        ApiRequest::Feeds => ApiResponsePayload::Feeds {
            feeds: vec![feed],
            feeds_groups: vec![],
        },
        ApiRequest::Items(_) |
        ApiRequest::ItemsSince(_) |
        ApiRequest::LatestItems => fetch_items(&feed),
        ApiRequest::UnreadItems => ApiResponsePayload::UnreadItems {
            unread_item_ids: vec![1],
        },
        _ => ApiResponsePayload::None,
    };
    ApiResponse {
        api_version: 1,
        auth: true,
        last_refreshed_on_time: None,
        payload: payload,
    }
}
