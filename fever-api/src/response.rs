use chrono::NaiveDateTime;
use serde::{self, Serialize};

use join_ids;

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
    // pub site_url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_spark: bool,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub last_updated_on_time: NaiveDateTime,
}

#[derive(Serialize)]
pub struct Item {
    pub id: u32,
    pub feed_id: u32,
    pub title: String,
    // pub author: String,
    pub html: String,
    pub url: String,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_saved: bool,
    #[serde(serialize_with = "serialize_bool_as_number")]
    pub is_read: bool,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub created_on_time: NaiveDateTime,
}

pub enum ApiResponsePayload {
    None,
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
        unread_item_ids: Vec<u32>,
    },
    SavedItems {
        saved_item_ids: Vec<u32>,
    },
}

impl ApiResponsePayload {
    fn num_fields(&self) -> usize {
        use self::ApiResponsePayload::*;

        match *self {
            None => 0,
            Groups {..} | Feeds {..} | Items {..} => 2,
            UnreadItems {..} | SavedItems {..} => 1,
        }
    }

    fn serialize_fields<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::ser::SerializeStruct {
        use self::ApiResponsePayload::*;

        match *self {
            Groups { ref groups, ref feeds_groups } => {
                serializer.serialize_field("groups", groups)?;
                serializer.serialize_field("feeds_groups", feeds_groups)
            },
            Feeds { ref feeds, ref feeds_groups } => {
                serializer.serialize_field("feeds", feeds)?;
                serializer.serialize_field("feeds_groups", feeds_groups)
            },
            Items { ref items, total_items } => {
                serializer.serialize_field("items", items)?;
                serializer.serialize_field("total_items", &total_items)
            },
            UnreadItems { ref unread_item_ids } => {
                let mut s = String::new();
                join_ids(unread_item_ids, &mut s);
                serializer.serialize_field("unread_item_ids", &s)
            },
            SavedItems { ref saved_item_ids } => {
                let mut s = String::new();
                join_ids(saved_item_ids, &mut s);
                serializer.serialize_field("saved_item_ids", &s)
            },
            None => Ok(())
        }
    }
}

pub struct ApiResponse {
    pub api_version: u32,
    pub auth: bool,
    pub last_refreshed_on_time: Option<NaiveDateTime>,
    pub payload: ApiResponsePayload,
}

impl Serialize for ApiResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeStruct;

        let num_fields = 2 + self.payload.num_fields() +
            if self.last_refreshed_on_time.is_some() {1} else {0};
        let mut s = serializer.serialize_struct("ApiResponse", num_fields)?;

        s.serialize_field("api_version", &self.api_version)?;
        let auth_num = if self.auth {1} else {0};
        s.serialize_field("auth", &auth_num)?;
        if let Some(refresh_time) = self.last_refreshed_on_time {
            let timestamp = refresh_time.timestamp();
            s.serialize_field("last_refreshed_on_time", &timestamp)?;
        }
        self.payload.serialize_fields(&mut s)?;

        s.end()
    }
}
