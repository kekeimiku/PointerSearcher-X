use std::{fmt::Display, io};

pub enum Error {
    #[cfg(feature = "dumper")]
    Vmmap(vmmap::Error),
    Io(io::Error),
    Other(String),
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Other(value.to_string())
    }
}

#[cfg(feature = "dumper")]
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
            #[cfg(feature = "dumper")]
            Error::Vmmap(err) => write!(f, "{err}"),
            Error::Other(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}
