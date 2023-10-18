#![allow(clippy::missing_safety_doc)]

#[cfg(not(target_endian = "little"))]
compile_error!("not supported.");

mod ffi_types;

use std::{
    ffi::{c_char, c_int, CStr, CString},
    fs::{File, OpenOptions},
    ops::Range,
    ptr,
    str::Utf8Error,
};

pub use ffi_types::*;
use ptrsx::PtrsxScanner;
use vmmap::Pid;

macro_rules! try_result {
    ($p:expr, $m:expr) => {
        match $m {
            Ok(val) => val,
            Err(err) => {
                $p.set_last_error(err);
                return -1;
            }
        }
    };
}

#[derive(Default)]
pub struct PointerSearcherX {
    inner: PtrsxScanner,
    last_error: Option<CString>,
    modules: Option<Vec<Module>>,
}

const PARAMS_ERROR: &str = "params error";

impl PointerSearcherX {
    fn set_last_error(&mut self, err: impl ToString) {
        self.last_error = Some(unsafe { CString::from_vec_unchecked(err.to_string().into()) })
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_last_error(ptr: *mut PointerSearcherX) -> *const c_char {
    let ptrsx = &(*ptr);
    match &ptrsx.last_error {
        Some(err) => err.as_ptr(),
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn ptrsx_init() -> *mut PointerSearcherX {
    Box::into_raw(Box::default())
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_free(ptr: *mut PointerSearcherX) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn create_pointer_map_file(
    ptr: *mut PointerSearcherX,
    pid: Pid,
    align: bool,
    info_file_path: *const c_char,
    bin_file_path: *const c_char,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let info_file_path = try_result!(ptrsx, CStr::from_ptr(info_file_path).to_str());
    let bin_file_path = try_result!(ptrsx, CStr::from_ptr(bin_file_path).to_str());
    let scanner = &ptrsx.inner;
    let info_file = try_result!(ptrsx, OpenOptions::new().append(true).create_new(true).open(info_file_path));
    let bin_file = try_result!(ptrsx, OpenOptions::new().append(true).create_new(true).open(bin_file_path));

    try_result!(ptrsx, scanner.create_pointer_map(pid, align, info_file, bin_file));

    0
}

#[no_mangle]
pub unsafe extern "C" fn load_pointer_map_file(
    ptr: *mut PointerSearcherX,
    bin_path: *const c_char,
    info_path: *const c_char,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &mut ptrsx.inner;
    let info_path = try_result!(ptrsx, CStr::from_ptr(info_path).to_str());
    let file = try_result!(ptrsx, File::open(info_path));
    let modules = try_result!(ptrsx, scanner.parse_modules_info(file))
        .into_iter()
        .map(|(Range { start, end }, name)| {
            let name = CString::new(name).unwrap().into_raw();
            Module { start, end, name }
        })
        .collect();
    ptrsx.modules = Some(modules);
    let bin_path = try_result!(ptrsx, CStr::from_ptr(bin_path).to_str());
    let file = try_result!(ptrsx, File::open(bin_path));
    try_result!(ptrsx, scanner.load_pointer_map(file));
    0
}

#[no_mangle]
pub unsafe extern "C" fn scanner_pointer_chain(
    ptr: *mut PointerSearcherX,
    modules: ModuleList,
    params: Params,
    file_path: *const c_char,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &mut ptrsx.inner;
    let Params { addr, depth, node, rangel, ranger } = params;
    if node >= depth {
        ptrsx.set_last_error(PARAMS_ERROR);
        return -1;
    }
    let file_name = try_result!(ptrsx, CStr::from_ptr(file_path).to_str());
    let file = try_result!(ptrsx, OpenOptions::new().append(true).create_new(true).open(file_name));

    let param = ptrsx::Param { depth, addr, node, range: (rangel, ranger) };
    let binding = &*ptr::slice_from_raw_parts(modules.data, modules.len);
    let modules = binding
        .iter()
        .map(|&Module { start, end, name }| Ok((start..end, CStr::from_ptr(name).to_str()?.to_string())))
        .collect::<Result<Vec<_>, Utf8Error>>();

    let modules = try_result!(ptrsx, modules);
    scanner.set_modules(modules.into_iter());

    try_result!(ptrsx, scanner.pointer_chain_scanner(param, file));

    0
}

#[no_mangle]
pub unsafe extern "C" fn get_modules_info(ptr: *mut PointerSearcherX) -> ModuleList {
    let modules = (*ptr).modules.as_ref().unwrap();
    let len = modules.len();
    let data = modules.as_ptr();
    ModuleList { len, data }
}
