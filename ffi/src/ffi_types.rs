use core::ffi;
use std::ffi::{CString, NulError};

use ptrsx::c64::Page;

#[repr(C)]
pub struct FFIPAGE {
    pub start: ffi::c_ulonglong,
    pub end: ffi::c_ulonglong,
    pub path: *const ffi::c_char,
}

impl Drop for FFIPAGE {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.path as _);
        }
    }
}

impl TryFrom<&Page<'_>> for FFIPAGE {
    type Error = NulError;

    fn try_from(value: &Page<'_>) -> Result<Self, Self::Error> {
        let path = CString::new(value.path)?.into_raw();
        Ok(Self { start: value.start, end: value.end, path })
    }
}

#[repr(C)]
pub struct Params {
    pub base: ffi::c_ulonglong,
    pub depth: ffi::c_ulonglong,
    pub ignore: ffi::c_ulonglong,
    pub rangel: ffi::c_ulonglong,
    pub ranger: ffi::c_ulonglong,
    pub target: ffi::c_ulonglong,
    pub out_dir: *const ffi::c_char,
}
