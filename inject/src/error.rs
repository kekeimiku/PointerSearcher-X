use std::fmt::Display;

use machx::kern_return::kern_return_t;

use super::ffi::mach_error;

pub enum Error {
    Kern(kern_return_t),
    Other(&'static str),
}

impl From<machx::kern_return::kern_return_t> for Error {
    fn from(value: kern_return_t) -> Self {
        Self::Kern(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Kern(err) => write!(f, "{}. code: {err}", unsafe { mach_error(*err) }),
            Error::Other(err) => write!(f, "{err}"),
        }
    }
}
