use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::feed::Feed;

pub fn load_feeds(conn: &PgConnection) -> QueryResult<Vec<Feed>> {
    Feed::load(conn)
}

pub fn load_unread_item_ids(conn: &PgConnection) -> QueryResult<Vec<i32>> {
    use schema::item::dsl::*;

    // TODO: filter out read items
    item.select(id)
        .load::<i32>(conn)
}

pub fn item_already_exists(link: &str, feed: &Feed, conn: &PgConnection)
-> QueryResult<bool> {
    use diesel::dsl::{exists, select};
    use schema::item::dsl::*;

    let query = item.filter(feed_id.eq(feed.id).and(url.eq(link)));
    select(exists(query))
        .get_result(conn)
}
