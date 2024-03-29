use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::item_identity::ItemIdentifier;
use crate::models::feed::Feed;
use crate::models::group::Group;
use crate::models::item::Item;

pub fn load_groups(conn: &mut PgConnection) -> QueryResult<Vec<Group>> {
    use crate::schema::feed_group::dsl::*;
    feed_group.load(conn)
}

pub fn load_feed_groups(conn: &mut PgConnection)
-> QueryResult<Vec<(i32, Option<i32>)>> {
    use crate::schema::feed::dsl::*;

    feed.select((id, group_id))
        .load(conn)
}

pub fn load_feeds(conn: &mut PgConnection) -> QueryResult<Vec<Feed>> {
    use crate::schema::feed::dsl::*;
    feed.load(conn)
}

pub enum ItemsQuery<'a> {
    Latest,
    Before(i32),
    After(i32),
    ForIds(&'a [i32]),
}

pub fn load_items(query_type: ItemsQuery, conn: &mut PgConnection)
-> QueryResult<Vec<Item>> {
    use crate::schema::item::dsl::*;

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

pub fn load_unread_item_ids(conn: &mut PgConnection) -> QueryResult<Vec<i32>> {
    use diesel::dsl::not;
    use crate::schema::item::dsl::*;

    item.filter(not(is_read))
        .select(id)
        .load::<i32>(conn)
}

pub fn load_saved_item_ids(conn: &mut PgConnection) -> QueryResult<Vec<i32>> {
    use crate::schema::item::dsl::*;

    item.filter(is_saved)
        .select(id)
        .load::<i32>(conn)
}

pub fn count_items(conn: &mut PgConnection) -> QueryResult<u32> {
    use crate::schema::item::dsl::*;

    let query = item.count();
    query.get_result::<i64>(conn).map(|i| i as u32)
}

pub fn load_latest_item_identifiers(feed: &Feed, conn: &mut PgConnection)
-> QueryResult<Vec<ItemIdentifier<'static>>> {
    use crate::schema::item;

    let results = item::table.filter(item::feed_id.eq(feed.id))
        .order(item::id.desc())
        .limit(10)
        .select((item::url, item::guid))
        .load::<(Option<String>, Option<String>)>(conn)?;

    let identifiers = results
        .into_iter()
        // Database ensures that both cannot be null, so this is safe
        .map(|(url, guid)| ItemIdentifier::new_owned(url, guid).unwrap())
        .collect();

    Ok(identifiers)
}

pub fn item_already_exists(
    identifier: &ItemIdentifier,
    feed: &Feed,
    conn: &mut PgConnection,
) -> QueryResult<bool> {
    use diesel::dsl::{exists, select};
    use crate::schema::item::dsl::*;

    // Compare insensitive to http vs https
    // some feeds seem to alternate...
    let http_link = identifier.link().map(|s| s.replace("https://", "http://"));
    let https_link = identifier.link().map(|s| s.replace("http://", "https://"));

    let identity_expr = url.eq(http_link)
        .or(url.eq(https_link))
        .or(guid.eq(identifier.guid()));

    let query = item.filter(feed_id.eq(feed.id).and(identity_expr));
    select(exists(query))
        .get_result(conn)
}

pub fn prune_read_items(conn: &mut PgConnection) -> QueryResult<usize> {
    let query = include_str!("prune.sql");
    diesel::sql_query(query)
        .execute(conn)
}
