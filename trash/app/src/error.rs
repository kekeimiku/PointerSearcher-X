#[derive(Debug)]
pub enum Error {
    IoError(::std::io::Error),
    ParseIntError(::core::num::ParseIntError),
}

impl From<::std::num::ParseIntError> for Error {
    fn from(value: ::std::num::ParseIntError) -> Self {
        Error::ParseIntError(value)
    }
}

impl From<::std::io::Error> for Error {
    fn from(value: ::std::io::Error) -> Self {
        Error::IoError(value)
    }
}

pub type Result<T, E = Error> = ::core::result::Result<T, E>;
