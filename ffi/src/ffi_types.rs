use core::ffi;
use std::ffi::CString;

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
