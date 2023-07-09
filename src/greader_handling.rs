use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::data;
use crate::error::Error;
use crate::greader::request::*;
use crate::greader::response::*;
use crate::models::feed::Feed as DbFeed;
use crate::models::group::Group as DbGroup;

type DataResult<T> = Result<T, Error<diesel::result::Error>>;

fn format_tag(group: DbGroup) -> Tag {
    Tag {
        id: StreamTag::Label(None, group.id.to_string()),
        sort_id: group.title,
    }
}

fn format_subscription(feed: DbFeed, group: Option<DbGroup>) -> Subscription {
    Subscription {
        title: feed.title,
        first_item_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        html_url: feed.site_url.unwrap(),
        sort_id: "".to_owned(),
        id: StreamId::Feed(feed.id.to_string()),
        categories: group.map(|group| {
            SubscriptionCategory {
                id: StreamTag::Label(None, group.id.to_string()),
                label: group.title,
            }
        }).into_iter().collect(),
    }
}

fn load_labels(conn: &mut PgConnection) -> DataResult<Vec<Tag>> {
    let labels = data::load_groups(conn)
        .map_err(fill_err!("Error loading groups"))?
        .into_iter()
        .map(format_tag)
        .collect();
    Ok(labels)
}

fn load_subscriptions(conn: &mut PgConnection) -> DataResult<Vec<Subscription>> {
    use crate::schema::{feed, feed_group};

    let subs = feed::table
        .left_join(feed_group::table)
        .select((DbFeed::as_select(), Option::<DbGroup>::as_select()))
        .load::<(DbFeed, Option<DbGroup>)>(conn)
        .map_err(fill_err!("Error loading feeds"))?
        .into_iter()
        .map(|(f, g)| format_subscription(f, g))
        .collect();
    Ok(subs)
}

#[derive(Clone, Debug)]
struct ItemsQuery {
    count: u32,
    descending: bool,
    continuing_from_id: Option<i32>,
    feed_id_filter: Option<i32>,
    read_state_filter: Option<bool>,
    saved_state_filter: Option<bool>,
    min_time: Option<NaiveDateTime>,
    max_time: Option<NaiveDateTime>,
}

impl ItemsQuery {
    fn new(
        stream_id: &StreamId,
        ranking: StreamRanking,
        number: u32,
        continuation: Option<&str>,
        exclude: Option<&StreamTag>,
        oldest_time: Option<NaiveDateTime>,
        newest_time: Option<NaiveDateTime>,
    ) -> Self {
        let feed_id_filter = match stream_id {
            StreamId::Feed(feed_id) => i32::from_str(feed_id).ok(),
            StreamId::Tag(_) => None,
        };

        let read_state_filter = match (stream_id, exclude) {
            (_, Some(StreamTag::State(_, StreamState::Read))) => Some(false),
            (_, Some(StreamTag::State(_, StreamState::KeptUnread))) => Some(true),
            (StreamId::Tag(StreamTag::State(_, StreamState::Read)), _) => Some(true),
            (StreamId::Tag(StreamTag::State(_, StreamState::KeptUnread)), _) => Some(false),
            _ => None,
        };

        let saved_state_filter = match (stream_id, exclude) {
            (_, Some(StreamTag::State(_, StreamState::Starred))) => Some(false),
            (StreamId::Tag(StreamTag::State(_, StreamState::Starred)), _) => Some(true),
            _ => None,
        };

        let count = if number == 0 { 20 } else { number };

        let descending = match ranking {
            StreamRanking::NewestFirst => true,
            StreamRanking::OldestFirst => false,
        };

        let continuing_from_id = continuation.and_then(|s| i32::from_str(&s).ok());

        ItemsQuery {
            count,
            descending,
            continuing_from_id,
            feed_id_filter,
            read_state_filter,
            saved_state_filter,
            min_time: oldest_time,
            max_time: newest_time,
        }
    }
}


fn load_item_ids(query: ItemsQuery) -> DataResult<StreamItemsIdsResponse> {
    Ok(StreamItemsIdsResponse {
        item_refs: vec![],
    })
}

pub fn handle_api_request(
    request: &RequestType,
    conn: &mut PgConnection,
) -> DataResult<Response> {
    use RequestType::*;

    let response: Response = match request {
        UserInfo => UserInfoResponse {
            user_id: "123".to_owned(),
            user_name: "Name".to_owned(),
            user_profile_id: "123".to_owned(),
            user_email: "username@gmail.com".to_owned(),
            is_blogger_user: true,
            signup_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            public_user_name: "username".to_owned(),
        }.into(),
        TagList => TagListResponse {
            tags: load_labels(conn)?,
        }.into(),
        SubscriptionList => SubscriptionListResponse {
            subscriptions: load_subscriptions(conn)?,
        }.into(),
        StreamItemsIds(params) => {
            let query = ItemsQuery::new(
                &params.stream_id,
                params.ranking,
                params.number,
                params.continuation.as_ref().map(String::as_str),
                params.exclude.as_ref(),
                params.oldest_time,
                params.newest_time,
            );
            load_item_ids(query)?.into()
        }
        _ => "OK".to_owned().into(),
    };

    Ok(response)
}
