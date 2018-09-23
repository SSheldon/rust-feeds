use chrono::NaiveDateTime;
use diesel::prelude::*;

use schema::item;
use super::feed::Feed;

mod queries {
    use diesel::helper_types::*;
    use schema::item;

    pub type DescendingItems = Limit<Order<item::table, Desc<item::columns::id>>>;
    pub type BeforeItems = Filter<DescendingItems, Lt<item::columns::id, i32>>;
    pub type AscendingItems = Limit<Order<item::table, Asc<item::columns::id>>>;
    pub type AfterItems = Filter<AscendingItems, Gt<item::columns::id, i32>>;
    pub type ForIds<'a> = Limit<Filter<item::table, EqAny<item::columns::id, &'a [i32]>>>;
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Feed)]
#[table_name = "item"]
pub struct Item {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub content: String,
    pub published: NaiveDateTime,
    pub feed_id: i32,
}

impl Item {
    const LIMIT: i64 = 50;

    pub fn latest_query() -> queries::DescendingItems {
        use schema::item::dsl::*;

        item.order(id.desc())
            .limit(Item::LIMIT)
    }

    pub fn earliest_query() -> queries::AscendingItems {
        use schema::item::dsl::*;

        item.order(id.asc())
            .limit(Item::LIMIT)
    }

    pub fn before_query(before_id: i32) -> queries::BeforeItems {
        use schema::item::dsl::*;

        Item::latest_query()
            .filter(id.lt(before_id))
    }

    pub fn after_query(after_id: i32) -> queries::AfterItems {
        use schema::item::dsl::*;

        Item::earliest_query()
            .filter(id.gt(after_id))
    }

    pub fn for_ids_query(ids: &[i32]) -> queries::ForIds {
        use schema::item::dsl::*;

        item.filter(id.eq_any(ids))
            .limit(Item::LIMIT)
    }
}

#[derive(Insertable)]
#[table_name="item"]
pub struct NewItem<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub published: Option<NaiveDateTime>,
    pub feed_id: i32,
}
