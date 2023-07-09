use std::str::FromStr;

use chrono::NaiveDateTime;

use super::request::ParseError;

fn to_timestamp_sec(i: i64) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::from_timestamp_opt(i, 0)
        .ok_or_else(|| ParseError { type_name: "TimestampSec", value: i.to_string() })
}

fn parse_timestamp_msec(s: String) -> Result<NaiveDateTime, ParseError> {
    i64::from_str(&s).ok()
        .and_then(NaiveDateTime::from_timestamp_millis)
        .ok_or_else(|| ParseError { type_name: "TimestampMSec", value: s })
}

fn parse_timestamp_usec(s: String) -> Result<NaiveDateTime, ParseError> {
    i64::from_str(&s).ok()
        .and_then(NaiveDateTime::from_timestamp_micros)
        .ok_or_else(|| ParseError { type_name: "TimestampMSec", value: s })
}

pub mod sec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serialize};

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let t = value.timestamp();
        t.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<NaiveDateTime, D::Error> where D: serde::Deserializer<'de> {
        i64::deserialize(deserializer).and_then(|i| {
            super::to_timestamp_sec(i)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod msec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize};

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let t = value.timestamp_millis();
        serializer.collect_str(&t)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<NaiveDateTime, D::Error> where D: serde::Deserializer<'de> {
        String::deserialize(deserializer).and_then(|s| {
            super::parse_timestamp_msec(s)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod usec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize};

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let t = value.timestamp_micros();
        serializer.collect_str(&t)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<NaiveDateTime, D::Error> where D: serde::Deserializer<'de> {
        String::deserialize(deserializer).and_then(|s| {
            super::parse_timestamp_usec(s)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod opt_sec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serialize};

    pub fn serialize<S>(value: &Option<NaiveDateTime>, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let t = value.as_ref().map(NaiveDateTime::timestamp);
        t.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<Option<NaiveDateTime>, D::Error> where D: serde::Deserializer<'de> {
        Option::<i64>::deserialize(deserializer).and_then(|i| {
            i.map(super::to_timestamp_sec)
                .transpose()
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod opt_msec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serialize};

    pub fn serialize<S>(value: &Option<NaiveDateTime>, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let s = value.as_ref()
            .map(NaiveDateTime::timestamp_millis)
            .map(|i| i.to_string());
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<Option<NaiveDateTime>, D::Error> where D: serde::Deserializer<'de> {
        Option::<String>::deserialize(deserializer).and_then(|s| {
            s.map(super::parse_timestamp_msec)
                .transpose()
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod opt_usec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serialize};

    pub fn serialize<S>(value: &Option<NaiveDateTime>, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let s = value.as_ref()
            .map(NaiveDateTime::timestamp_micros)
            .map(|i| i.to_string());
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D)
    -> Result<Option<NaiveDateTime>, D::Error> where D: serde::Deserializer<'de> {
        Option::<String>::deserialize(deserializer).and_then(|s| {
            s.map(super::parse_timestamp_usec)
                .transpose()
                .map_err(serde::de::Error::custom)
        })
    }
}
