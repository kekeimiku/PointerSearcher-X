use core::ffi::CStr;

use machx::{error::mach_error_string, kern_return::kern_return_t};

use super::QueryProcError;

#[derive(Debug)]
pub enum Error {
    AttachProcess(kern_return_t),
    QueryProcess(QueryProcError),
    ReadMemory(kern_return_t),
    Io(std::io::Error),
}

impl From<QueryProcError> for Error {
    fn from(value: QueryProcError) -> Self {
        Self::QueryProcess(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::AttachProcess(err) => {
                write!(
                    f,
                    "code: {err}, msg: {}",
                    unsafe { CStr::from_ptr(mach_error_string(*err)) }.to_string_lossy()
                )
            }
            Error::ReadMemory(err) => {
                write!(
                    f,
                    "code: {err}, msg: {}",
                    unsafe { CStr::from_ptr(mach_error_string(*err)) }.to_string_lossy()
                )
            }
            Error::QueryProcess(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}
