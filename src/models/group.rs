use crate::schema::feed_group;

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = feed_group)]
pub struct Group {
    pub id: i32,
    pub title: String,
}
