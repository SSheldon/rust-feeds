use chrono::NaiveDateTime;

use crate::schema::item;
use super::feed::Feed;

#[derive(Identifiable, Queryable, Associations)]
#[diesel(belongs_to(Feed))]
#[diesel(table_name = item)]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
    pub feed_id: i32,
    pub is_read: bool,
    pub is_saved: bool,
    pub author: Option<String>,
    pub fetched: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = item)]
pub struct NewItem<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub published: Option<NaiveDateTime>,
    pub feed_id: i32,
    pub author: Option<&'a str>,
}
