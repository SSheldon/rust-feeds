use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::Item as ApiItem;

use schema::item;
use super::feed::Feed;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Feed)]
#[table_name = "item"]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
    pub feed_id: i32,
}

impl Item {
    pub fn query(conn: &PgConnection,
            after_id: Option<i32>,
            before_id: Option<i32>)
            -> QueryResult<Vec<Item>> {
        use schema::item::dsl::*;

        let mut query = item.order(id.desc())
            .limit(5)
            .into_boxed();

        if let Some(after_id) = after_id {
            query = query.filter(id.gt(after_id));
        }
        if let Some(before_id) = before_id {
            query = query.filter(id.lt(before_id));
        }

        query.load::<Item>(conn)
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
