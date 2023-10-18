#![allow(clippy::missing_safety_doc)]

#[cfg(not(any(target_endian = "little", target_os = "android", target_arch = "x86")))]
panic!("not supported.");

mod ffi_types;

use std::{
    ffi::{c_char, c_int, CStr, CString},
    fs::OpenOptions,
    io::BufWriter,
    ops::Deref,
    path::Path,
    ptr, str,
};

pub use ffi_types::*;
use ptrsx::PtrsxScanner;
use vmmap::{Pid, Process};

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
    modules: Option<Vec<Module>>,
    last_error: Option<CString>,
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
    file_name: *const c_char,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let string = try_result!(ptrsx, str::from_utf8(CStr::from_ptr(file_name).to_bytes()));
    let file_name = Path::new(string);
    let scanner = &ptrsx.inner;
    let mut writer = BufWriter::new(try_result!(
        ptrsx,
        OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create_new(true)
            .open(file_name)
    ));

    try_result!(ptrsx, scanner.create_pointer_map_file(&mut writer, pid, align));

    0
}

#[no_mangle]
pub unsafe extern "C" fn create_pointer_map(ptr: *mut PointerSearcherX, pid: Pid, align: bool) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &mut ptrsx.inner;
    let proc = try_result!(ptrsx, Process::open(pid));
    try_result!(ptrsx, scanner.create_pointer_map(&proc, align));
    ptrsx.modules = Some(
        scanner
            .modules
            .iter()
            .map(|m| Module {
                start: m.start,
                end: m.end,
                name: CString::new(m.name.deref()).unwrap().into_raw(),
            })
            .collect(),
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn load_pointer_map_file(ptr: *mut PointerSearcherX, file_name: *mut c_char) -> c_int {
    let ptrsx = &mut (*ptr);
    let string = try_result!(ptrsx, str::from_utf8(CStr::from_ptr(file_name).to_bytes()));
    let path = Path::new(string);
    let scanner = &mut ptrsx.inner;
    try_result!(ptrsx, scanner.load_pointer_map_file(path));
    ptrsx.modules = Some(
        scanner
            .modules
            .iter()
            .map(|m| Module {
                start: m.start,
                end: m.end,
                name: CString::new(m.name.deref()).unwrap().into_raw(),
            })
            .collect(),
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn get_modules(ptr: *mut PointerSearcherX) -> Modules {
    let modules = (*ptr).modules.as_ref().unwrap();
    let len = modules.len();
    let data = modules.as_ptr();
    Modules { len, data }
}

#[no_mangle]
pub unsafe extern "C" fn scanner_pointer_chain_with_module(
    ptr: *mut PointerSearcherX,
    module: Module,
    params: Params,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &ptrsx.inner;
    let Params { target, depth, node, rangel, ranger, file_name } = params;
    if node >= depth || depth > 32 {
        ptrsx.set_last_error(PARAMS_ERROR);
        return -1;
    }
    let string = try_result!(ptrsx, str::from_utf8(CStr::from_ptr(file_name).to_bytes()));
    let file_name = Path::new(string);
    let mut writer = BufWriter::new(try_result!(
        ptrsx,
        OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create_new(true)
            .open(file_name)
    ));

    let module = ptrsx::Module { start: module.start, end: module.end, ..Default::default() };
    #[rustfmt::skip]
    let params = ptrsx::Params {
        depth, target, node,
        offset: (rangel, ranger),
        writer: &mut writer,
    };

    try_result!(ptrsx, scanner.scanner_with_module(&module, params));

    0
}

#[no_mangle]
pub unsafe extern "C" fn scanner_pointer_chain_with_address(
    ptr: *mut PointerSearcherX,
    address: usize,
    params: Params,
) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &ptrsx.inner;
    let Params { target, depth, node, rangel, ranger, file_name } = params;
    if node >= depth || depth > 32 {
        ptrsx.set_last_error(PARAMS_ERROR);
        return -1;
    }
    let string = try_result!(ptrsx, str::from_utf8(CStr::from_ptr(file_name).to_bytes()));
    let file_name = Path::new(string);
    let mut writer = BufWriter::new(try_result!(
        ptrsx,
        OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create_new(true)
            .open(file_name)
    ));
    #[rustfmt::skip]
    let params = ptrsx::Params {
        depth, target, node,
        offset: (rangel, ranger),
        writer: &mut writer,
    };
    try_result!(ptrsx, scanner.scanner_with_address(address, params));

    0
}
