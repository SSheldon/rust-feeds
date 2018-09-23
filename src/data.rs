use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::feed::Feed;
use models::item::Item;
use models::read::Read;

mod queries {
    use diesel::helper_types::*;
    use schema::{item, read};

    type ReadProfileExpr = Or<Eq<read::profile_id, i32>, IsNull<read::profile_id>>;
    pub type ItemReads = Filter<LeftJoin<item::table, read::table>, ReadProfileExpr>;
}

fn item_reads_query(profile_id: i32) -> queries::ItemReads {
    use schema::{item, read};

    let expr = read::profile_id.eq(profile_id).or(read::profile_id.is_null());
    item::table.left_join(read::table).filter(expr)
}

pub fn get_profile_id(api_key: &str, conn: &PgConnection)
-> QueryResult<Option<i32>> {
    use schema::profile::dsl::*;

    profile.filter(key.eq(api_key))
        .select(id)
        .get_result(conn)
        .optional()
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

pub fn load_items(query_type: ItemsQuery, profile_id: i32, conn: &PgConnection)
-> QueryResult<Vec<(Item, Option<Read>)>> {
    use schema::item::dsl::*;

    let query = item_reads_query(profile_id).limit(50);
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

pub fn load_unread_item_ids(profile_id: i32, conn: &PgConnection) -> QueryResult<Vec<i32>> {
    use diesel::dsl::not;
    use schema::item::dsl::*;
    use schema::read;

    item_reads_query(profile_id)
        .filter(not(read::is_read).or(read::is_read.is_null()))
        .select(id)
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
