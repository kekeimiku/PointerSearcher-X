use core::ffi;
use std::{
    ffi::{CStr, CString, OsStr},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
};

use dumper::map::Map;

#[repr(C)]
#[derive(Clone)]
pub struct Addr {
    pub start: usize,
    pub end: usize,
    // references to ptrsx.map
    // SAFETY: DO NOT use after ptrsx.free()! thus results use-after-free
    pub path: *const ffi::c_char,
}

impl Drop for Addr {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.path as _);
        }
    }
}

impl Into<Map> for &Addr {
    fn into(self) -> Map {
        Map {
            start: self.start,
            end: self.end,
            path: PathBuf::from(OsStr::from_bytes(unsafe { CStr::from_ptr(self.path) }.to_bytes())),
        }
    }
}
