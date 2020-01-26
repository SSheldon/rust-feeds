use crate::schema::feed;
use super::group::Group;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Group)]
#[table_name = "feed"]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub group_id: Option<i32>,
    pub site_url: Option<String>,
}

#[derive(Insertable)]
#[table_name="feed"]
pub struct NewFeed<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub site_url: Option<&'a str>,
}
