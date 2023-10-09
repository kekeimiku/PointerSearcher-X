use std::ffi::{c_char, c_int, CString};

// vmmap Pid
pub type Pid = c_int;

#[repr(C)]
pub struct Module {
    pub start: usize,
    pub end: usize,
    pub name: *mut c_char,
}

#[repr(C)]
pub struct Modules {
    pub len: usize,
    pub data: *const Module,
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.name);
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
    pub file_name: *const c_char,
}
