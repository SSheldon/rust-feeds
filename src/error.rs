use std::error::Error as StdError;
use std::fmt;

use diesel;

#[derive(Debug)]
pub struct Error {
    pub description: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub source: ErrorSource,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: {} ({})", self.file, self.line, self.description, self.source)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.source {
            ErrorSource::Data(ref err) => Some(err),
            ErrorSource::Conn(ref err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum ErrorSource {
    Data(diesel::result::Error),
    Conn(diesel::r2d2::PoolError),
}

impl fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorSource::Data(err) => fmt::Display::fmt(err, f),
            ErrorSource::Conn(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl From<diesel::result::Error> for ErrorSource {
    fn from(error: diesel::result::Error) -> Self {
        ErrorSource::Data(error)
    }
}

impl From<diesel::r2d2::PoolError> for ErrorSource {
    fn from(error: diesel::r2d2::PoolError) -> Self {
        ErrorSource::Conn(error)
    }
}

macro_rules! fill_err {
    ($msg:expr) => ({
        |err| Error {
            description: $msg,
            file: file!(),
            line: line!(),
            source: err.into(),
        }
    })
}
