use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::Feed as ApiFeed;

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

    pub fn into_api_feed(self) -> ApiFeed {
        ApiFeed {
            id: self.id as u32,
            title: self.title,
            url: self.url,
            is_spark: false,
            last_updated_on_time: NaiveDateTime::from_timestamp(1472799906, 0),
        }
    }
}

#[derive(Insertable)]
#[table_name="feed"]
pub struct NewFeed<'a> {
    pub url: &'a str,
    pub title: &'a str,
}
