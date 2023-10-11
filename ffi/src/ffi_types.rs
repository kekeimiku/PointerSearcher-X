use core::ffi::{c_char, c_int};

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

#[repr(C)]
pub struct Params {
    pub target: usize,
    pub depth: usize,
    pub node: usize,
    pub rangel: usize,
    pub ranger: usize,
    pub file_name: *const c_char,
}
