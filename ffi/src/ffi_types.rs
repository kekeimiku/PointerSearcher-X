use core::ffi;
use std::ffi::CString;

#[repr(C)]
pub struct Addr {
    pub start: *const ffi::c_void,
    pub end: *const ffi::c_void,
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
