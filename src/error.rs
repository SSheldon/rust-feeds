use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct Error<C> {
    pub description: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub source: C,
}

impl<C: fmt::Display> fmt::Display for Error<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: {} ({})", self.file, self.line, self.description, self.source)
    }
}

impl<C: 'static + StdError> StdError for Error<C> {
    fn source(&self) -> Option<&(StdError + 'static)> {
        Some(&self.source)
    }
}

macro_rules! fill_err {
    ($msg:expr) => ({
        |err| Error {
            description: $msg,
            file: file!(),
            line: line!(),
            source: err,
        }
    })
}
