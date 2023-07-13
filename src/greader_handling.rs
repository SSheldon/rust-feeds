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
use crate::models::item::Item as DbItem;

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

type BoxedItemFilter = Box<dyn diesel::expression::BoxableExpression<
    crate::schema::item::table,
    diesel::pg::Pg,
    SqlType = diesel::sql_types::Bool
>>;

macro_rules! filter_and {
    ($base:expr, $new:expr) => {
        if let Some(e) = $base.take() {
            $base = Some(Box::new(e.and($new)));
        } else {
            $base = Some(Box::new($new));
        }
    };
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

    fn expr(&self) -> Option<BoxedItemFilter> {
        use crate::schema::item;

        let mut expr: Option<BoxedItemFilter> = None;

        if let Some(id) = self.continuing_from_id {
            if self.descending {
                filter_and!(expr, item::id.lt(id));
            } else {
                filter_and!(expr, item::id.gt(id));
            }
        }
        if let Some(feed_id) = self.feed_id_filter {
            filter_and!(expr, item::feed_id.eq(feed_id));
        }
        if let Some(is_read) = self.read_state_filter {
            filter_and!(expr, item::is_read.eq(is_read));
        }
        if let Some(is_saved) = self.saved_state_filter {
            filter_and!(expr, item::is_saved.eq(is_saved));
        }
        if let Some(min_time) = self.min_time {
            filter_and!(expr, item::published.ge(min_time));
        }
        if let Some(max_time) = self.max_time {
            filter_and!(expr, item::published.le(max_time));
        }

        expr
    }
}

fn load_item_ids(query: ItemsQuery, conn: &mut PgConnection) -> DataResult<StreamItemsIdsResponse> {
    use crate::schema::item;

    let ids = if let Some(expr) = query.expr() {
        item::table.filter(expr)
            .select((item::id, item::published))
            .load::<(i32, NaiveDateTime)>(conn)
    } else {
        item::table.select((item::id, item::published))
            .load::<(i32, NaiveDateTime)>(conn)
    }.map_err(fill_err!("Error loading item ids"))?;

    let refs = ids.into_iter()
        .map(|(id, published)| {
            ItemRef {
                id: ItemId(id as u64),
                timestamp: published,
                direct_stream_ids: vec![],
            }
        })
        .collect();

    Ok(StreamItemsIdsResponse {
        item_refs: refs,
    })
}

fn load_items_for_ids(ids: &[ItemId], conn: &mut PgConnection) -> DataResult<StreamItemsContentsResponse> {
    use crate::schema::item;

    let db_ids: Vec<i32> = ids.iter().map(|&i| i.0 as i32).collect();

    let db_items = item::table.filter(item::id.eq_any(db_ids))
        .load::<DbItem>(conn)
        .map_err(fill_err!("Error loading items"))?;

    let api_items = db_items.into_iter()
        .map(|item| {
            Item {
                origin: ItemOrigin {
                    stream_id: StreamId::Feed(item.feed_id.to_string()),
                },
                updated: item.published,
                id: ItemId(item.id as u64),
                categories: vec![],
                author: "".to_owned(),
                alternate: vec![
                    Link {
                        href: item.url,
                        link_type: None,
                    },
                ],
                timestamp: item.published,
                summary: ItemSummary {
                    direction: "ltr".to_owned(),
                    content: item.content,
                },
                crawl_time: item.published,
                published: item.published,
                title: item.title,
            }
        })
        .collect();

    Ok(StreamItemsContentsResponse {
        items: api_items,
    })
}

pub fn handle_api_request(
    request: &Request,
    conn: &mut PgConnection,
) -> DataResult<Response> {
    use Request::*;

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
            load_item_ids(query, conn)?.into()
        }
        StreamItemsContents(params) => load_items_for_ids(&params.item_ids, conn)?.into(),
        _ => "OK".to_owned().into(),
    };

    Ok(response)
}
