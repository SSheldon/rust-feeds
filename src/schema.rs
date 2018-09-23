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

table! {
    profile (id) {
        id -> Int4,
        name -> Varchar,
        key -> Varchar,
    }
}

table! {
    read (profile_id, item_id) {
        profile_id -> Int4,
        item_id -> Int4,
        is_read -> Bool,
        is_saved -> Bool,
    }
}

joinable!(item -> feed (feed_id));
joinable!(read -> item (item_id));
joinable!(read -> profile (profile_id));

allow_tables_to_appear_in_same_query!(
    feed,
    item,
    profile,
    read,
);
