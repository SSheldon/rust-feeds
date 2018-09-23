use chrono::NaiveDateTime;

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
    pub is_read: bool,
    pub is_saved: bool,
}

#[derive(Insertable)]
#[table_name="item"]
pub struct NewItem<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub published: Option<NaiveDateTime>,
    pub feed_id: i32,
}
