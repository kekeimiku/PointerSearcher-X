use core::ffi::c_char;

#[repr(C)]
pub struct Module {
    pub start: usize,
    pub end: usize,
    pub name: *mut c_char,
}

#[repr(C)]
pub struct ModuleList {
    pub len: usize,
    pub data: *const Module,
}

#[repr(C)]
pub struct Params {
    pub addr: usize,
    pub depth: usize,
    pub node: usize,
    pub rangel: usize,
    pub ranger: usize,
}
