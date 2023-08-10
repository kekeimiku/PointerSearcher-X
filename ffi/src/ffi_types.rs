use std::ffi::{c_char, CString};

#[repr(C)]
pub struct Page {
    pub start: usize,
    pub end: usize,
    pub path: *mut c_char,
}

#[repr(C)]
pub struct PageVec {
    pub len: usize,
    pub data: *const Page,
}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.path);
        }
    }
}

#[repr(C)]
pub struct Params {
    pub target: usize,
    pub depth: usize,
    pub node: usize,
    pub rangel: usize,
    pub ranger: usize,
    pub dir: *const c_char,
}
