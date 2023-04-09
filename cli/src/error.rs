#[derive(Debug)]
pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self(value.to_string())
    }
}

impl From<ptrsx::Error> for Error {
    fn from(value: ptrsx::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
