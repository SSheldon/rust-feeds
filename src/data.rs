use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::feed::Feed;
use models::item::Item;

pub fn load_feeds(conn: &PgConnection) -> QueryResult<Vec<Feed>> {
    use schema::feed::dsl::*;
    feed.load(conn)
}

pub enum ItemsQuery<'a> {
    Latest,
    Before(i32),
    After(i32),
    ForIds(&'a [i32]),
}

pub fn load_items(query_type: ItemsQuery, conn: &PgConnection)
-> QueryResult<Vec<Item>> {
    use schema::item::dsl::*;

    let query = item.limit(50);
    match query_type {
        ItemsQuery::Latest => {
            query.order(id.desc())
                .load(conn)
        },
        ItemsQuery::Before(before_id) => {
            query.order(id.desc())
                .filter(id.lt(before_id))
                .load(conn)
        },
        ItemsQuery::After(after_id) => {
            query.order(id.asc())
                .filter(id.gt(after_id))
                .load(conn)
        },
        ItemsQuery::ForIds(ids) => {
            query.filter(id.eq_any(ids))
                .load(conn)
        },
    }
}

pub fn load_unread_item_ids(conn: &PgConnection) -> QueryResult<Vec<i32>> {
    use schema::item::dsl::*;

    // TODO: filter out read items
    item.select(id)
        .load::<i32>(conn)
}

pub fn count_items(conn: &PgConnection) -> QueryResult<u32> {
    use schema::item::dsl::*;

    let query = item.count();
    query.get_result::<i64>(conn).map(|i| i as u32)
}

pub fn item_already_exists(link: &str, feed: &Feed, conn: &PgConnection)
-> QueryResult<bool> {
    use diesel::dsl::{exists, select};
    use schema::item::dsl::*;

    let query = item.filter(feed_id.eq(feed.id).and(url.eq(link)));
    select(exists(query))
        .get_result(conn)
}
