use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Vmmap(vmmap::Error),
    Io(io::Error),
    Other(String),
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self::Vmmap(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Vmmap(err) => write!(f, "{err}"),
            Error::Other(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}
