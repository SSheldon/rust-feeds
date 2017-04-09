use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::Item as ApiItem;

use schema::item;

#[derive(Queryable)]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
}

impl Item {
    pub fn query(conn: &PgConnection) -> QueryResult<Vec<Item>> {
        use schema::item::dsl::*;

        item.limit(5)
            .load::<Item>(conn)
    }

    pub fn into_api_item(self, feed_id: u32) -> ApiItem {
        ApiItem {
            id: self.id as u32,
            feed_id: feed_id,
            title: self.title,
            url: self.url,
            html: self.content,
            is_saved: false,
            is_read: false,
            created_on_time: self.published,
        }
    }
}

#[derive(Insertable)]
#[table_name="item"]
pub struct NewItem<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub published: Option<NaiveDateTime>,
}
