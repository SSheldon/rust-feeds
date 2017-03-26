use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
}
