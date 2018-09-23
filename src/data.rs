use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::feed::Feed;
use models::item::Item;

mod queries {
    use diesel::helper_types::*;
    use schema::item;

    pub type DescendingItems = Limit<Order<item::table, Desc<item::columns::id>>>;
    pub type BeforeItems = Filter<DescendingItems, Lt<item::columns::id, i32>>;
    pub type AscendingItems = Limit<Order<item::table, Asc<item::columns::id>>>;
    pub type AfterItems = Filter<AscendingItems, Gt<item::columns::id, i32>>;
    pub type ForIds<'a> = Limit<Filter<item::table, EqAny<item::columns::id, &'a [i32]>>>;
}

const ITEM_LIMIT: i64 = 50;

fn latest_items_query() -> queries::DescendingItems {
    use schema::item::dsl::*;

    item.order(id.desc())
        .limit(ITEM_LIMIT)
}

fn earliest_items_query() -> queries::AscendingItems {
    use schema::item::dsl::*;

    item.order(id.asc())
        .limit(ITEM_LIMIT)
}

fn items_before_query(before_id: i32) -> queries::BeforeItems {
    use schema::item::dsl::*;

    latest_items_query()
        .filter(id.lt(before_id))
}

fn items_after_query(after_id: i32) -> queries::AfterItems {
    use schema::item::dsl::*;

    earliest_items_query()
        .filter(id.gt(after_id))
}

fn items_for_ids_query(ids: &[i32]) -> queries::ForIds {
    use schema::item::dsl::*;

    item.filter(id.eq_any(ids))
        .limit(ITEM_LIMIT)
}

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

pub fn load_items(query: ItemsQuery, conn: &PgConnection)
-> QueryResult<Vec<Item>> {
    match query {
        ItemsQuery::Latest => latest_items_query().load(conn),
        ItemsQuery::Before(id) => items_before_query(id).load(conn),
        ItemsQuery::After(id) => items_after_query(id).load(conn),
        ItemsQuery::ForIds(ids) => items_for_ids_query(ids).load(conn),
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
