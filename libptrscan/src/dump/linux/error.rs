#[derive(Debug)]
pub enum Error {
    AttachProcess(std::io::Error),
    QueryProcess(std::io::Error),
    ReadMemory(std::io::Error),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::AttachProcess(err) => write!(f, "{err}"),
            Error::QueryProcess(err) => write!(f, "{err}"),
            Error::ReadMemory(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}
