use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{self, Deserialize, Serialize};

use super::request::ParseError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampSec(pub NaiveDateTime);

impl TryFrom<i64> for TimestampSec {
    type Error = ParseError;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        NaiveDateTime::from_timestamp_opt(i, 0)
            .map(TimestampSec)
            .ok_or_else(|| ParseError { type_name: "TimestampSec", value: i.to_string() })
    }
}

impl<'de> Deserialize<'de> for TimestampSec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let i = i64::deserialize(deserializer)?;
        TimestampSec::try_from(i).map_err(serde::de::Error::custom)
    }
}

impl Serialize for TimestampSec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampMSec(pub NaiveDateTime);

impl fmt::Display for TimestampMSec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.timestamp_millis().fmt(f)
    }
}

impl FromStr for TimestampMSec {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        i64::from_str(s).ok()
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(TimestampMSec)
            .ok_or_else(|| ParseError { type_name: "TimestampMSec", value: s.to_owned() })
    }
}

impl<'de> Deserialize<'de> for TimestampMSec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for TimestampMSec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampUSec(pub NaiveDateTime);

impl fmt::Display for TimestampUSec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.timestamp_micros().fmt(f)
    }
}

impl FromStr for TimestampUSec {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        i64::from_str(s).ok()
            .and_then(NaiveDateTime::from_timestamp_micros)
            .map(TimestampUSec)
            .ok_or_else(|| ParseError { type_name: "TimestampUSec", value: s.to_owned() })
    }
}

impl<'de> Deserialize<'de> for TimestampUSec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for TimestampUSec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

