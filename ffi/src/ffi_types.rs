use std::ffi::{c_char, CStr};

use ptrsx::Page;

#[repr(C)]
pub struct FFIPAGE {
    pub start: usize,
    pub end: usize,
    pub path: *const c_char,
}

impl From<&Page<'_>> for FFIPAGE {
    fn from(value: &Page<'_>) -> Self {
        let path = value.path.as_ptr() as _;
        Self { start: value.start, end: value.end, path }
    }
}

// TODO ub
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
    pub depth: usize,
    pub node: usize,
    pub rangel: usize,
    pub ranger: usize,
    pub target: usize,
    pub out_dir: *const c_char,
}
