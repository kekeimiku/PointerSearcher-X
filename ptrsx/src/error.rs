#[derive(Debug)]
pub enum Error {
    Vm(vmmap::Error),
    Io(std::io::Error),
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self::Vm(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Vm(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = core::result::Result<T, E>;
