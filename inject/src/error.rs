use std::fmt::Display;

use machx::kern_return::kern_return_t;

#[derive(Debug)]
pub enum Error {
    Kern(kern_return_t),
    Other(String),
}

impl From<machx::kern_return::kern_return_t> for Error {
    fn from(value: kern_return_t) -> Self {
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
