use chrono::NaiveDateTime;
use serde::{self, Serialize};
use serde_derive::Serialize;

use crate::join_ids;

fn serialize_bool_as_number<S>(value: &bool, serializer: S)
        -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
    let i = if *value {1} else {0};
    i.serialize(serializer)
}

fn serialize_datetime_as_timestamp<S>(value: &NaiveDateTime, serializer: S)
        -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
    let t = value.timestamp();
    t.serialize(serializer)
}

fn serialize_opt_datetime_as_timestamp<S>(value: &Option<NaiveDateTime>, serializer: S)
        -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
    let t = value.as_ref().map(NaiveDateTime::timestamp);
    t.serialize(serializer)
}

fn serialize_ids_as_comma_string<S>(value: &[u32], serializer: S)
        -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
    let mut s = String::new();
    join_ids(value, &mut s);
    s.serialize(serializer)
}

#[derive(Serialize)]
pub struct Group {
    pub id: u32,
    pub title: String,
}

#[derive(Serialize)]
pub struct FeedsGroup {
    pub group_id: u32,
    #[serde(serialize_with = "serialize_ids_as_comma_string")]
    pub feed_ids: Vec<u32>,
}

#[derive(Serialize)]
pub struct Feed {
    pub id: u32,
    // pub favicon_id: u32,
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_url: Option<String>,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_spark: bool,
    #[serde(skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_opt_datetime_as_timestamp")]
    pub last_updated_on_time: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct Item {
    pub id: u32,
    pub feed_id: u32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub html: String,
    pub url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_saved: bool,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_read: bool,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub created_on_time: NaiveDateTime,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ResponsePayload {
    None {},
    Groups {
        groups: Vec<Group>,
        feeds_groups: Vec<FeedsGroup>,
    },
    Feeds {
        feeds: Vec<Feed>,
        feeds_groups: Vec<FeedsGroup>,
    },
    Items {
        items: Vec<Item>,
        total_items: u32,
    },
    UnreadItems {
        #[serde(serialize_with = "serialize_ids_as_comma_string")]
        unread_item_ids: Vec<u32>,
    },
    SavedItems {
        #[serde(serialize_with = "serialize_ids_as_comma_string")]
        saved_item_ids: Vec<u32>,
    },
}

#[derive(Serialize)]
pub struct Response {
    pub api_version: u32,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub auth: bool,
    #[serde(skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_opt_datetime_as_timestamp")]
    pub last_refreshed_on_time: Option<NaiveDateTime>,
    #[serde(flatten)]
    pub payload: ResponsePayload,
}
