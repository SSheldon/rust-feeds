table! {
    feed (id) {
        id -> Int4,
        url -> Varchar,
        title -> Varchar,
        group_id -> Nullable<Int4>,
    }
}

table! {
    feed_group (id) {
        id -> Int4,
        title -> Varchar,
    }
}

table! {
    item (id) {
        id -> Int4,
        url -> Varchar,
        title -> Varchar,
        content -> Text,
        published -> Timestamp,
        feed_id -> Int4,
        is_read -> Bool,
        is_saved -> Bool,
    }
}

joinable!(feed -> feed_group (group_id));
joinable!(item -> feed (feed_id));

allow_tables_to_appear_in_same_query!(
    feed,
    feed_group,
    item,
);
