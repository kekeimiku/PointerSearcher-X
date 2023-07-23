use std::ffi::{self, CStr};

use ptrsx::c64::Page;

#[repr(C)]
pub struct FFIPAGE {
    pub start: ffi::c_ulonglong,
    pub end: ffi::c_ulonglong,
    pub path: *const ffi::c_char,
}

impl From<&Page<'_>> for FFIPAGE {
    fn from(value: &Page<'_>) -> Self {
        let path = value.path.as_ptr() as _;
        Self { start: value.start, end: value.end, path }
    }
}

impl From<&FFIPAGE> for Page<'_> {
    fn from(value: &FFIPAGE) -> Self {
        unsafe {
            let path = std::str::from_utf8_unchecked(CStr::from_ptr(value.path).to_bytes());
            Self { start: value.start, end: value.end, path }
        }
    }
}

#[repr(C)]
pub struct FFIParams {
    pub depth: ffi::c_ulonglong,
    pub rangel: ffi::c_ulonglong,
    pub ranger: ffi::c_ulonglong,
    pub target: ffi::c_ulonglong,
    pub out_dir: *const ffi::c_char,
}
