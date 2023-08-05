// @generated automatically by Diesel CLI.

diesel::table! {
    feed (id) {
        id -> Int4,
        url -> Varchar,
        title -> Varchar,
        group_id -> Nullable<Int4>,
        site_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    feed_group (id) {
        id -> Int4,
        title -> Varchar,
    }
}

diesel::table! {
    item (id) {
        id -> Int4,
        url -> Varchar,
        title -> Varchar,
        content -> Text,
        published -> Timestamp,
        feed_id -> Int4,
        is_read -> Bool,
        is_saved -> Bool,
        author -> Nullable<Varchar>,
        fetched -> Timestamp,
        guid -> Nullable<Varchar>,
    }
}

diesel::joinable!(feed -> feed_group (group_id));
diesel::joinable!(item -> feed (feed_id));

diesel::allow_tables_to_appear_in_same_query!(
    feed,
    feed_group,
    item,
);
