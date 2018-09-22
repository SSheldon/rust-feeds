use schema::profile;

#[derive(Identifiable, Queryable)]
#[table_name = "profile"]
pub struct Profile {
    pub id: i32,
    pub name: String,
    pub key: String,
}
