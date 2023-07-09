use std::marker::PhantomData;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{self, Deserialize, Serialize};

use super::request::ParseError;

pub trait Convert<T> {
    type Value: Serialize + for<'a> Deserialize<'a>;

    fn timestamp_from_value(value: Self::Value) -> Result<T, ParseError>;
    fn timestamp_as_value(timestamp: &T) -> Self::Value;

    fn serialize<S>(timestamp: &T, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        Self::timestamp_as_value(timestamp).serialize(serializer)
    }

    fn deserialize<'de, D>(deserializer: D)
    -> Result<T, D::Error> where D: serde::Deserializer<'de> {
        Self::Value::deserialize(deserializer).and_then(|v| {
            Self::timestamp_from_value(v)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub enum Sec {}

impl Convert<NaiveDateTime> for Sec {
    type Value = i64;

    fn timestamp_from_value(i: i64) -> Result<NaiveDateTime, ParseError> {
        NaiveDateTime::from_timestamp_opt(i, 0)
            .ok_or_else(|| ParseError { type_name: "Sec", value: i.to_string() })
    }

    fn timestamp_as_value(timestamp: &NaiveDateTime) -> i64 {
        timestamp.timestamp()
    }
}

pub enum MSec {}

impl Convert<NaiveDateTime> for MSec {
    type Value = String;

    fn timestamp_from_value(s: String) -> Result<NaiveDateTime, ParseError> {
        i64::from_str(&s).ok()
            .and_then(NaiveDateTime::from_timestamp_millis)
            .ok_or_else(|| ParseError { type_name: "MSec", value: s })
    }

    fn timestamp_as_value(timestamp: &NaiveDateTime) -> String {
        timestamp.timestamp_millis().to_string()
    }
}

pub enum USec {}

impl Convert<NaiveDateTime> for USec {
    type Value = String;

    fn timestamp_from_value(s: String) -> Result<NaiveDateTime, ParseError> {
        i64::from_str(&s).ok()
            .and_then(NaiveDateTime::from_timestamp_micros)
            .ok_or_else(|| ParseError { type_name: "USec", value: s })
    }

    fn timestamp_as_value(timestamp: &NaiveDateTime) -> String {
        timestamp.timestamp_micros().to_string()
    }
}

pub struct Opt<T>(PhantomData<T>);

impl<T> Convert<Option<NaiveDateTime>> for Opt<T>
where T: Convert<NaiveDateTime> {
    type Value = Option<T::Value>;

    fn timestamp_from_value(v: Self::Value) -> Result<Option<NaiveDateTime>, ParseError> {
        v.map(T::timestamp_from_value).transpose()
    }

    fn timestamp_as_value(timestamp: &Option<NaiveDateTime>) -> Self::Value {
        timestamp.as_ref().map(T::timestamp_as_value)
    }
}

pub type OptSec = Opt<Sec>;
pub type OptMSec = Opt<MSec>;
pub type OptUSec = Opt<USec>;
