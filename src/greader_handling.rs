use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::pg::{Pg, PgConnection};

use crate::data;
use crate::error::Error;
use crate::greader::request::*;
use crate::greader::response::*;
use crate::models::feed::Feed as DbFeed;
use crate::models::group::Group as DbGroup;
use crate::models::item::Item as DbItem;
use crate::schema::{feed, feed_group, item};

type DataResult<T> = Result<T, Error<diesel::result::Error>>;

fn format_tag(group: DbGroup) -> Tag {
    Tag {
        id: StreamTag::Label(None, group.title.clone()),
        sort_id: None,
    }
}

fn format_subscription(feed: DbFeed, group: Option<DbGroup>) -> Subscription {
    Subscription {
        id: StreamId::Feed(feed.id.to_string()),
        title: feed.title,
        html_url: feed.site_url,
        sort_id: None,
        categories: group.map(|group| {
            SubscriptionCategory {
                id: StreamTag::Label(None, group.title),
                label: None,
            }
        }).into_iter().collect(),
    }
}

fn format_item(item: DbItem) -> Item {
    Item {
        id: ItemId(item.id as i64),
        origin: ItemOrigin {
            stream_id: StreamId::Feed(item.feed_id.to_string()),
        },
        categories: [
            if item.is_read { Some(StreamTag::State(None, StreamState::Read)) } else { None },
            if item.is_saved { Some(StreamTag::State(None, StreamState::Starred)) } else { None },
        ].into_iter().filter_map(|t| t).collect(),
        alternate: vec![
            Link { href: item.url },
        ],
        author: item.author,
        title: item.title,
        summary: ItemSummary {
            content: item.content,
        },
        timestamp: item.published,
        published: item.published,
        updated: None,
        crawl_time: None,
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

type BoxedItemExpr = Box<dyn BoxableExpression<item::table, Pg, SqlType = diesel::sql_types::Bool>>;

fn merge<T, F>(x: Option<T>, y: Option<T>, f: F) -> Option<T>
where F: FnOnce(T, T) -> T {
    match (x, y) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (x, y) => x.or(y)
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct ItemsFilter {
    feed_id: Option<i32>,
    read_state: Option<bool>,
    saved_state: Option<bool>,
    min_time: Option<NaiveDateTime>,
    max_time: Option<NaiveDateTime>,
}

impl ItemsFilter {
    fn stream(stream_id: &StreamId) -> Self {
        match stream_id {
            StreamId::Tag(tag) => Self::tag(tag),
            StreamId::Feed(feed_id) => Self::feed(feed_id),
        }
    }

    fn feed(feed_id: &str) -> Self {
        ItemsFilter {
            feed_id: i32::from_str(feed_id).ok(),
            ..Self::default()
        }
    }

    fn tag(tag: &StreamTag) -> Self {
        match *tag {
            StreamTag::State(_, state) => Self::state(state),
            StreamTag::Label(_, _) => Self::default(),
        }
    }

    fn state(state: StreamState) -> Self {
        match state {
            StreamState::Read => ItemsFilter {
                read_state: Some(true),
                ..Self::default()
            },
            StreamState::KeptUnread => ItemsFilter {
                read_state: Some(false),
                ..Self::default()
            },
            StreamState::Starred => ItemsFilter {
                saved_state: Some(true),
                ..Self::default()
            },
            StreamState::ReadingList => Self::default(),
        }
    }

    fn exclude_tag(tag: &StreamTag) -> Self {
        match *tag {
            StreamTag::State(_, state) => Self::exclude_state(state),
            StreamTag::Label(_, _) => Self::default(),
        }
    }

    fn exclude_state(state: StreamState) -> Self {
        match state {
            StreamState::Read => ItemsFilter {
                read_state: Some(false),
                ..Self::default()
            },
            StreamState::KeptUnread => ItemsFilter {
                read_state: Some(true),
                ..Self::default()
            },
            StreamState::Starred => ItemsFilter {
                saved_state: Some(false),
                ..Self::default()
            },
            StreamState::ReadingList => Self::default(),
        }
    }

    fn and(&self, other: ItemsFilter) -> Self {
        ItemsFilter {
            feed_id: other.feed_id.or(self.feed_id),
            read_state: other.read_state.or(self.read_state),
            saved_state: other.saved_state.or(self.saved_state),
            min_time: merge(other.min_time, self.min_time, std::cmp::max),
            max_time: merge(other.max_time, self.max_time, std::cmp::min),
        }
    }

    fn new(
        stream_id: &StreamId,
        exclude: Option<&StreamTag>,
        oldest_time: Option<NaiveDateTime>,
        newest_time: Option<NaiveDateTime>,
    ) -> Self {
        let mut filter = Self::stream(stream_id);
        if let Some(tag) = exclude {
            filter = filter.and(Self::exclude_tag(tag));
        }
        filter.min_time = oldest_time;
        filter.max_time = newest_time;
        filter
    }

    fn expr(&self) -> Option<BoxedItemExpr> {
        [
            self.feed_id.map::<BoxedItemExpr, _>(|feed_id| {
                Box::new(item::feed_id.eq(feed_id))
            }),
            self.read_state.map::<BoxedItemExpr, _>(|is_read| {
                Box::new(item::is_read.eq(is_read))
            }),
            self.saved_state.map::<BoxedItemExpr, _>(|is_saved| {
                Box::new(item::is_saved.eq(is_saved))
            }),
            self.min_time.map::<BoxedItemExpr, _>(|min_time| {
                Box::new(item::published.ge(min_time))
            }),
            self.max_time.map::<BoxedItemExpr, _>(|max_time| {
                Box::new(item::published.le(max_time))
            }),
        ].into_iter().fold(None, |x, y| {
            merge(x, y, |a, b| Box::new(a.and(b)))
        })
    }

    fn query(&self) -> item::BoxedQuery<Pg> {
        let mut query = item::table.into_boxed();

        if let Some(expr) = self.expr() {
            query = query.filter(expr);
        }

        query
    }
}

#[derive(Clone, Debug)]
struct ItemsQuery {
    count: u32,
    descending: bool,
    continuing_from_id: Option<i32>,
    filter: ItemsFilter,
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
        let filter = ItemsFilter::new(
            stream_id,
            exclude,
            oldest_time,
            newest_time,
        );

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
            filter,
        }
    }

    fn query(&self) -> item::BoxedQuery<Pg> {
        let mut query = self.filter.query()
            .limit(self.count as i64);

        if self.descending {
            query = query.order(item::id.desc());
        } else {
            query = query.order(item::id.asc());
        }

        if let Some(id) = self.continuing_from_id {
            if self.descending {
                query = query.filter(item::id.le(id));
            } else {
                query = query.filter(item::id.ge(id));
            }
        }

        query
    }
}

fn load_item_ids(query: ItemsQuery, conn: &mut PgConnection) -> DataResult<StreamItemsIdsResponse> {
    let ids = query.query()
        .select((item::id, item::published))
        .load::<(i32, NaiveDateTime)>(conn)
        .map_err(fill_err!("Error loading item ids"))?;

    let continuation = query.query()
        .offset(query.count as i64)
        .limit(1)
        .select(item::id)
        .get_result::<i32>(conn)
        .optional()
        .map_err(fill_err!("Error loading item ids continuation"))?
        .map(|id| id.to_string());

    let refs = ids.into_iter()
        .map(|(id, published)| {
            ItemRef {
                id: ItemId(id as i64),
                timestamp: published,
                direct_stream_ids: vec![],
            }
        })
        .collect();

    Ok(StreamItemsIdsResponse {
        item_refs: refs,
        continuation: continuation,
    })
}

fn load_items_for_ids(ids: &[ItemId], conn: &mut PgConnection) -> DataResult<StreamItemsContentsResponse> {
    let db_ids: Vec<i32> = ids.iter().map(|&i| i.0 as i32).collect();

    let db_items = item::table.filter(item::id.eq_any(db_ids))
        .load::<DbItem>(conn)
        .map_err(fill_err!("Error loading items"))?;

    let api_items = db_items.into_iter()
        .map(format_item)
        .collect();

    Ok(StreamItemsContentsResponse {
        items: api_items,
    })
}

fn load_items_for_stream(
    stream_id: &StreamId,
    query: ItemsQuery,
    conn: &mut PgConnection,
) -> DataResult<StreamContentsResponse> {
    let feed = if let Some(feed_id) = query.filter.feed_id {
        feed::table.filter(feed::id.eq(feed_id))
            .get_result::<DbFeed>(conn)
            .optional()
            .map_err(fill_err!("Error loading feed"))?
    } else { None };

    let (feed_title, feed_url) = match feed {
        Some(feed) => (Some(feed.title), feed.site_url),
        None => (None, None),
    };

    let db_items = query.query()
        .load::<DbItem>(conn)
        .map_err(fill_err!("Error loading items"))?;

    let continuation = query.query()
        .offset(query.count as i64)
        .limit(1)
        .select(item::id)
        .get_result::<i32>(conn)
        .optional()
        .map_err(fill_err!("Error loading item ids continuation"))?
        .map(|id| id.to_string());

    let api_items = db_items.into_iter()
        .map(format_item)
        .collect();

    Ok(StreamContentsResponse {
        id: stream_id.clone(),
        title: feed_title,
        alternate: feed_url.map(|url| Link { href: url }).into_iter().collect(),
        updated: None,
        items: api_items,
        continuation: continuation,
    })
}

fn load_item_count(filter: ItemsFilter, conn: &mut PgConnection) -> DataResult<u32> {
    filter.query()
        .count()
        .get_result::<i64>(conn)
        .map_err(fill_err!("Error counting items"))
        .map(|i| i as u32)
}

fn load_unread_counts(conn: &mut PgConnection) -> DataResult<UnreadCountResponse> {
    use diesel::dsl::{count, max};

    let counts = feed::table.inner_join(item::table)
        .filter(item::is_read.eq(false))
        .group_by(feed::id)
        .select((feed::id, count(item::id), max(item::published)))
        .load::<(i32, i64, Option<NaiveDateTime>)>(conn)
        .map_err(fill_err!("Error counting unread items"))?;

    let counts = counts
        .into_iter()
        .map(|(id, count, latest)| {
            UnreadCount {
                id: StreamId::Feed(id.to_string()),
                count: count as u32,
                newest_item_time: latest,
            }
        })
        .collect();

    Ok(UnreadCountResponse {
        unread_counts: counts,
    })
}

fn update_items_tags(params: &EditTagParams, conn: &mut PgConnection) -> DataResult<()> {
    let db_ids: Vec<i32> = params.item_ids.iter().map(|&i| i.0 as i32).collect();

    let mut read_state: Option<bool> = None;
    let mut saved_state: Option<bool> = None;

    for tag in &params.add_tags {
        match tag {
            StreamTag::State(_, StreamState::Read) => read_state = Some(true),
            StreamTag::State(_, StreamState::KeptUnread) => read_state = Some(false),
            StreamTag::State(_, StreamState::Starred) => saved_state = Some(true),
            StreamTag::State(_, StreamState::ReadingList) => (),
            StreamTag::Label(_, _) => (),
        }
    }

    for tag in &params.remove_tags {
        match tag {
            StreamTag::State(_, StreamState::Read) => read_state = Some(false),
            StreamTag::State(_, StreamState::KeptUnread) => read_state = Some(true),
            StreamTag::State(_, StreamState::Starred) => saved_state = Some(false),
            StreamTag::State(_, StreamState::ReadingList) => (),
            StreamTag::Label(_, _) => (),
        }
    }

    let query = diesel::update(item::table)
        .filter(item::id.eq_any(db_ids));

    match (read_state, saved_state) {
        (Some(is_read), Some(is_saved)) => {
            query.set((item::is_read.eq(is_read), item::is_saved.eq(is_saved)))
                .execute(conn)
                .map_err(fill_err!("Error updating items"))?;
        }
        (Some(is_read), None) => {
            query.set(item::is_read.eq(is_read))
                .execute(conn)
                .map_err(fill_err!("Error updating items"))?;
        }
        (None, Some(is_saved)) => {
            query.set(item::is_saved.eq(is_saved))
                .execute(conn)
                .map_err(fill_err!("Error updating items"))?;
        }
        (None, None) => (),
    }

    Ok(())
}

fn update_stream_read(params: &MarkAllAsReadParams, conn: &mut PgConnection) -> DataResult<()> {
    let mut filter = ItemsFilter::stream(&params.stream_id);
    filter.max_time = params.older_than;

    let mut query = diesel::update(item::table)
        .set(item::is_read.eq(true))
        .into_boxed();

    if let Some(expr) = filter.expr() {
        query = query.filter(expr);
    }

    query
        .execute(conn)
        .map_err(fill_err!("Error marking stream read"))?;

    Ok(())
}

pub fn handle_api_request(
    request: &Request,
    conn: &mut PgConnection,
) -> DataResult<Response> {
    use Request::*;

    let response: Response = match request {
        Token => "TOKEN".to_owned().into(),
        UserInfo => UserInfoResponse {
            user_id: "1".to_owned(),
            user_profile_id: "1".to_owned(),
            user_name: "user".to_owned(),
            user_email: None,
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
        StreamContents(stream_id, params) => {
            let query = ItemsQuery::new(
                &stream_id,
                params.ranking,
                params.number,
                params.continuation.as_ref().map(String::as_str),
                params.exclude.as_ref(),
                params.oldest_time,
                params.newest_time,
            );
            load_items_for_stream(stream_id, query, conn)?.into()
        }
        StreamItemsCount(params) => {
            let filter = ItemsFilter::new(
                &params.stream_id,
                params.exclude.as_ref(),
                params.oldest_time,
                params.newest_time,
            );
            load_item_count(filter, conn)?.to_string().into()
        }
        UnreadCount => load_unread_counts(conn)?.into(),
        EditTag(params) => {
            update_items_tags(params, conn)?;
            "OK".to_owned().into()
        }
        MarkAllAsRead(params) => {
            update_stream_read(params, conn)?;
            "OK".to_owned().into()
        },
    };

    Ok(response)
}
