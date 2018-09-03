table! {
    feed (id) {
        id -> Int4,
        url -> Varchar,
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
    }
}

joinable!(item -> feed (feed_id));

allow_tables_to_appear_in_same_query!(
    feed,
    item,
);
