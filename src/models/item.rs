use chrono::NaiveDateTime;

use fever_api::Item as ApiItem;

#[derive(Queryable)]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
}

impl Item {
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
