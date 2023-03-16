#[derive(Debug)]
pub enum Error {
    ParseInt(std::num::ParseIntError),
    Inquire(inquire::InquireError),
    Io(std::io::Error),
    Vmmap(super::vmmap::error::Error),
    Fmt(std::fmt::Error),
    Other(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<super::vmmap::error::Error> for Error {
    fn from(value: super::vmmap::error::Error) -> Self {
        Self::Vmmap(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<inquire::InquireError> for Error {
    fn from(value: inquire::InquireError) -> Self {
        Self::Inquire(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::Fmt(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Other(String::from(value))
    }
}
