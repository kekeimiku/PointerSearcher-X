use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Process(vmmap::Error),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Other(String),
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self::Process(value)
    }
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

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::Process(e) => e.to_string(),
            Error::Io(e) => e.to_string(),
            Error::Utf8(e) => e.to_string(),
            Error::Other(e) => e.to_string(),
        };

        write!(f, "{s}")
    }
}

impl std::error::Error for Error {}
