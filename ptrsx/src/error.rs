use std::fmt::Display;

pub enum Error {
    Vmmap(vmmap::Error),
    Io(std::io::Error),
    Other(String),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
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

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::Vmmap(err) => format!("Vmmap: {err}"),
            Error::Other(err) => format!("Other: {err}"),
            Error::Io(err) => format!("Io: {err}"),
        };
        write!(f, "{s}")
    }
}
