use crate::schema::feed;
use super::group::Group;

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Group))]
#[diesel(table_name = feed)]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub group_id: Option<i32>,
    pub site_url: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = feed)]
pub struct NewFeed<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub site_url: Option<&'a str>,
}
