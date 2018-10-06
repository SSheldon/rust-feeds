use schema::feed_group;

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "feed_group"]
pub struct Group {
    pub id: i32,
    pub title: String,
}
