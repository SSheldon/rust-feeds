use std::collections::HashMap;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::{
    Key as ApiKey,
    Request as ApiRequest,
    RequestType as ApiRequestType,
    Response as ApiResponse,
    ResponsePayload as ApiResponsePayload,
    self,
};

use data::{ItemsQuery, self};
use error::Error;
use models::feed::Feed as DbFeed;
use models::group::Group as DbGroup;
use models::item::Item as DbItem;

type DataError = Error<diesel::result::Error>;
type DataResult<T> = Result<T, DataError>;

fn format_group(group: DbGroup) -> fever_api::Group {
    fever_api::Group {
        id: group.id as u32,
        title: group.title,
    }
}

fn format_feed(feed: DbFeed) -> fever_api::Feed {
    fever_api::Feed {
        id: feed.id as u32,
        title: feed.title,
        url: feed.url,
        site_url: feed.site_url,
        is_spark: false,
        last_updated_on_time: None,
    }
}

fn format_feeds_groups(feed_groups: impl Iterator<Item=(i32, Option<i32>)>)
-> Vec<fever_api::FeedsGroup> {
    let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();
    for (feed_id, group_id) in feed_groups {
        if let Some(group_id) = group_id {
            groups.entry(group_id as u32).or_default().push(feed_id as u32)
        }
    }

    groups.into_iter()
        .map(|(group_id, feed_ids)| fever_api::FeedsGroup { group_id, feed_ids })
        .collect()
}

fn format_item(item: DbItem) -> fever_api::Item {
    fever_api::Item {
        id: item.id as u32,
        feed_id: item.feed_id as u32,
        title: item.title,
        author: None,
        url: item.url,
        html: item.content,
        is_saved: item.is_saved,
        is_read: item.is_read,
        created_on_time: item.published,
    }
}

fn load_groups(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let groups = data::load_groups(conn)
        .map_err(fill_err!("Error loading groups"))?
        .into_iter()
        .map(format_group)
        .collect();

    let feed_groups = data::load_feed_groups(conn)
        .map_err(fill_err!("Error loading feeds"))?;
    let feeds_groups = format_feeds_groups(feed_groups.into_iter());

    Ok(ApiResponsePayload::Groups { groups, feeds_groups })
}

fn load_feeds(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let feeds = data::load_feeds(conn)
        .map_err(fill_err!("Error loading feeds"))?;

    let feeds_groups = {
        let feed_groups = feeds.iter()
            .map(|feed| (feed.id, feed.group_id));

        format_feeds_groups(feed_groups)
    };

    let feeds = feeds
        .into_iter()
        .map(format_feed)
        .collect();

    Ok(ApiResponsePayload::Feeds {
        feeds: feeds,
        feeds_groups: feeds_groups,
    })
}

fn load_items(query: ItemsQuery, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    let items = data::load_items(query, conn)
        .map_err(fill_err!("Error loading items"))?
        .into_iter()
        .map(format_item)
        .collect();
    let total_items = data::count_items(conn)
        .map_err(fill_err!("Error counting items"))?;

    Ok(ApiResponsePayload::Items {
        items: items,
        total_items: total_items,
    })
}

fn load_unread_item_ids(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let ids = data::load_unread_item_ids(conn)
        .map_err(fill_err!("Error loading unread item ids"))?
        .into_iter()
        .map(|i| i as u32)
        .collect();

    Ok(ApiResponsePayload::UnreadItems {
        unread_item_ids: ids,
    })
}

fn load_saved_item_ids(conn: &PgConnection) -> DataResult<ApiResponsePayload> {
    let ids = data::load_saved_item_ids(conn)
        .map_err(fill_err!("Error loading saved item ids"))?
        .into_iter()
        .map(|i| i as u32)
        .collect();

    Ok(ApiResponsePayload::SavedItems {
        saved_item_ids: ids,
    })
}

fn update_item_read(id: u32, is_read: bool, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_read.eq(is_read))
        .execute(conn)
        .map_err(fill_err!("Error updating item is_read"))?;

    load_unread_item_ids(conn)
}

fn update_item_saved(id: u32, is_saved: bool, conn: &PgConnection)
-> DataResult<ApiResponsePayload> {
    use schema::item;

    diesel::update(item::table.find(id as i32))
        .set(item::is_saved.eq(is_saved))
        .execute(conn)
        .map_err(fill_err!("Error updating item is_saved"))?;

    load_saved_item_ids(conn)
}

pub fn handle_api_request(
    request: &ApiRequest,
    expected_key: Option<&ApiKey>,
    conn: &PgConnection,
) -> DataResult<ApiResponse> {
    let mut response = ApiResponse {
        api_version: 1,
        auth: false,
        last_refreshed_on_time: None,
        payload: ApiResponsePayload::None {},
    };

    if !expected_key.map_or(true, |key| request.api_key == *key) {
        return Ok(response);
    }
    response.auth = true;

    response.payload = match request.req_type {
        ApiRequestType::Groups => load_groups(conn)?,
        ApiRequestType::Feeds => load_feeds(conn)?,
        ApiRequestType::LatestItems => {
            load_items(ItemsQuery::Latest, conn)?
        },
        ApiRequestType::ItemsBefore(id) => {
            load_items(ItemsQuery::Before(id as i32), conn)?
        },
        ApiRequestType::ItemsSince(id) => {
            load_items(ItemsQuery::After(id as i32), conn)?
        },
        ApiRequestType::Items(ref ids) => {
            let ids: Vec<_> = ids.iter().map(|&i| i as i32).collect();
            load_items(ItemsQuery::ForIds(&ids), conn)?
        }
        ApiRequestType::UnreadItems => load_unread_item_ids(conn)?,
        ApiRequestType::MarkItemRead(id) => update_item_read(id, true, conn)?,
        ApiRequestType::MarkItemUnread(id) => update_item_read(id, false, conn)?,
        ApiRequestType::MarkItemSaved(id) => update_item_saved(id, true, conn)?,
        ApiRequestType::MarkItemUnsaved(id) => update_item_saved(id, false, conn)?,
        _ => ApiResponsePayload::None {},
    };

    Ok(response)
}
