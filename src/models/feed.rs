use diesel::prelude::*;
use diesel::pg::PgConnection;

use schema::feed;

#[derive(Identifiable, Queryable)]
#[table_name = "feed"]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
}

impl Feed {
    pub fn load(conn: &PgConnection) -> QueryResult<Vec<Feed>> {
        use schema::feed::dsl::*;
        feed.load(conn)
    }
}

#[derive(Insertable)]
#[table_name="feed"]
pub struct NewFeed<'a> {
    pub url: &'a str,
    pub title: &'a str,
}
