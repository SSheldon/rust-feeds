use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use fever_api::Item as ApiItem;

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

    pub fn count(conn: &PgConnection) -> QueryResult<u32> {
        use schema::item::dsl::*;

        let query = item.count();
        query.get_result::<i64>(conn).map(|i| i as u32)
    }

    pub fn into_api_item(self) -> ApiItem {
        ApiItem {
            id: self.id as u32,
            feed_id: self.feed_id as u32,
            title: self.title,
            url: self.url,
            html: self.content,
            is_saved: false,
            is_read: false,
            created_on_time: self.published,
        }
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
