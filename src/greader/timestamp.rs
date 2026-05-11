use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};

use super::request::ParseError;

pub trait Transform {
    type Value;
    type Raw: Serialize + for<'a> Deserialize<'a>;
    type Error: Display;

    fn from_raw(raw: Self::Raw) -> Result<Self::Value, Self::Error>;
    fn to_raw(value: &Self::Value) -> Self::Raw;

    fn serialize<S>(value: &Self::Value, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        Self::to_raw(value).serialize(serializer)
    }

    fn deserialize<'de, D>(deserializer: D)
    -> Result<Self::Value, D::Error> where D: serde::Deserializer<'de> {
        Self::Raw::deserialize(deserializer).and_then(|raw| {
            Self::from_raw(raw)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub enum Sec {}

impl Transform for Sec {
    type Value = DateTime<Utc>;
    type Raw = i64;
    type Error = ParseError;

    fn from_raw(i: i64) -> Result<DateTime<Utc>, ParseError> {
        DateTime::from_timestamp_secs(i)
            .ok_or_else(|| ParseError { type_name: "Sec", value: i.to_string() })
    }

    fn to_raw(timestamp: &DateTime<Utc>) -> i64 {
        timestamp.timestamp()
    }
}

pub enum MSec {}

impl Transform for MSec {
    type Value = DateTime<Utc>;
    type Raw = String;
    type Error = ParseError;

    fn from_raw(s: String) -> Result<DateTime<Utc>, ParseError> {
        i64::from_str(&s).ok()
            .and_then(DateTime::from_timestamp_millis)
            .ok_or_else(|| ParseError { type_name: "MSec", value: s })
    }

    fn to_raw(timestamp: &DateTime<Utc>) -> String {
        timestamp.timestamp_millis().to_string()
    }
}

pub enum USec {}

impl Transform for USec {
    type Value = DateTime<Utc>;
    type Raw = String;
    type Error = ParseError;

    fn from_raw(s: String) -> Result<DateTime<Utc>, ParseError> {
        i64::from_str(&s).ok()
            .and_then(DateTime::from_timestamp_micros)
            .ok_or_else(|| ParseError { type_name: "USec", value: s })
    }

    fn to_raw(timestamp: &DateTime<Utc>) -> String {
        timestamp.timestamp_micros().to_string()
    }
}

pub struct Opt<T>(PhantomData<T>);

impl<T: Transform> Transform for Opt<T> {
    type Value = Option<T::Value>;
    type Raw = Option<T::Raw>;
    type Error = T::Error;

    fn from_raw(raw: Self::Raw) -> Result<Self::Value, Self::Error> {
        raw.map(T::from_raw).transpose()
    }

    fn to_raw(value: &Self::Value) -> Self::Raw {
        value.as_ref().map(T::to_raw)
    }
}

pub type OptSec = Opt<Sec>;
pub type OptMSec = Opt<MSec>;
pub type OptUSec = Opt<USec>;
