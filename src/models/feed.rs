use schema::feed;

#[derive(Identifiable, Queryable)]
#[table_name = "feed"]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
}

#[derive(Insertable)]
#[table_name="feed"]
pub struct NewFeed<'a> {
    pub url: &'a str,
    pub title: &'a str,
}
