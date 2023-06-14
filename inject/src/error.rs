use std::fmt::Display;

use super::bindgen;

#[derive(Debug)]
pub enum Error {
    Kern(bindgen::kern_return_t),
    Other(String),
}

impl From<bindgen::kern_return_t> for Error {
    fn from(value: bindgen::kern_return_t) -> Self {
        Self::Kern(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Other(String::from(value))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::Kern(err) => format!("Kern: {err}"),
            Error::Other(err) => format!("other: {err}"),
        };
        write!(f, "{s}")
    }
}

impl std::error::Error for Error {}
