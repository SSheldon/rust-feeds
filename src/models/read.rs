use schema::read;
use super::profile::Profile;
use super::item::Item;

#[derive(Queryable, Associations)]
#[belongs_to(Profile)]
#[belongs_to(Item)]
#[table_name = "read"]
pub struct Read {
    pub profile_id: i32,
    pub item_id: i32,
    pub is_read: bool,
    pub is_saved: bool,
}
