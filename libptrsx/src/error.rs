#[derive(Debug)]
pub struct Error(String);

impl ToString for Error {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self(value.to_string())
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self(value.to_string())
    }
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self(value.to_string())
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
