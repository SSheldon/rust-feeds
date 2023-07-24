use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{self, Deserialize, Serialize};

use super::request::ParseError;

pub trait Convert<T> {
    type Raw: Serialize + for<'a> Deserialize<'a>;
    type Error: Display;

    fn from_raw(raw: Self::Raw) -> Result<T, Self::Error>;
    fn to_raw(value: &T) -> Self::Raw;

    fn serialize<S>(value: &T, serializer: S)
    -> Result<S::Ok, S::Error> where S: serde::Serializer {
        Self::to_raw(value).serialize(serializer)
    }

    fn deserialize<'de, D>(deserializer: D)
    -> Result<T, D::Error> where D: serde::Deserializer<'de> {
        Self::Raw::deserialize(deserializer).and_then(|raw| {
            Self::from_raw(raw)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub enum Sec {}

impl Convert<NaiveDateTime> for Sec {
    type Raw = i64;
    type Error = ParseError;

    fn from_raw(i: i64) -> Result<NaiveDateTime, ParseError> {
        NaiveDateTime::from_timestamp_opt(i, 0)
            .ok_or_else(|| ParseError { type_name: "Sec", value: i.to_string() })
    }

    fn to_raw(timestamp: &NaiveDateTime) -> i64 {
        timestamp.timestamp()
    }
}

pub enum MSec {}

impl Convert<NaiveDateTime> for MSec {
    type Raw = String;
    type Error = ParseError;

    fn from_raw(s: String) -> Result<NaiveDateTime, ParseError> {
        i64::from_str(&s).ok()
            .and_then(NaiveDateTime::from_timestamp_millis)
            .ok_or_else(|| ParseError { type_name: "MSec", value: s })
    }

    fn to_raw(timestamp: &NaiveDateTime) -> String {
        timestamp.timestamp_millis().to_string()
    }
}

pub enum USec {}

impl Convert<NaiveDateTime> for USec {
    type Raw = String;
    type Error = ParseError;

    fn from_raw(s: String) -> Result<NaiveDateTime, ParseError> {
        i64::from_str(&s).ok()
            .and_then(NaiveDateTime::from_timestamp_micros)
            .ok_or_else(|| ParseError { type_name: "USec", value: s })
    }

    fn to_raw(timestamp: &NaiveDateTime) -> String {
        timestamp.timestamp_micros().to_string()
    }
}

pub struct Opt<T>(PhantomData<T>);

impl<T> Convert<Option<NaiveDateTime>> for Opt<T>
where T: Convert<NaiveDateTime> {
    type Raw = Option<T::Raw>;
    type Error = T::Error;

    fn from_raw(raw: Self::Raw) -> Result<Option<NaiveDateTime>, Self::Error> {
        raw.map(T::from_raw).transpose()
    }

    fn to_raw(value: &Option<NaiveDateTime>) -> Self::Raw {
        value.as_ref().map(T::to_raw)
    }
}

pub type OptSec = Opt<Sec>;
pub type OptMSec = Opt<MSec>;
pub type OptUSec = Opt<USec>;
