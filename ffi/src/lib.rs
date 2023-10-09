#![allow(clippy::missing_safety_doc)]

mod error;
mod ffi_types;

use std::{
    ffi::{c_char, c_int, CStr, CString, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    path::Path,
};

pub use error::*;
pub use ffi_types::*;
use ptrsx::PtrsxScanner;
use vmmap::{Pid, Process};

#[derive(Default)]
pub struct PointerSearcherX {
    inner: PtrsxScanner,
    modules: Option<Vec<Module>>,
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
    file_name: *const c_char,
) -> c_int {
    let file_name = Path::new(OsStr::from_bytes(CStr::from_ptr(file_name).to_bytes()));
    let ptrsx = &(*ptr).inner;
    let mut writer = BufWriter::new(ffi_try_result!(
        OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create_new(true)
            .open(file_name),
        -1
    ));

    ffi_try_result!(ptrsx.create_pointer_map_file(&mut writer, pid, true), -1);

    0
}

#[no_mangle]
pub unsafe extern "C" fn create_pointer_map(ptr: *mut PointerSearcherX, pid: Pid) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &mut ptrsx.inner;
    let proc = ffi_try_result!(Process::open(pid), -1);
    ffi_try_result!(scanner.create_pointer_map(&proc, true), -1);
    ptrsx.modules = Some(
        scanner
            .modules
            .iter()
            .map(|m| Module {
                start: m.start,
                end: m.end,
                name: CString::new(&*m.name).unwrap().into_raw(),
            })
            .collect(),
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn load_pointer_map_file(ptr: *mut PointerSearcherX, file_name: *mut c_char) -> c_int {
    let ptrsx = &mut (*ptr);
    let scanner = &mut ptrsx.inner;
    let path = Path::new(OsStr::from_bytes(CStr::from_ptr(file_name).to_bytes()));
    ffi_try_result!(scanner.load_pointer_map_file(path), -1);
    ptrsx.modules = Some(
        scanner
            .modules
            .iter()
            .map(|m| Module {
                start: m.start,
                end: m.end,
                name: CString::new(&*m.name).unwrap().into_raw(),
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
    let scanner = &(*ptr).inner;
    let Params { target, depth, node, rangel, ranger, file_name } = params;
    let file_name = Path::new(OsStr::from_bytes(CStr::from_ptr(file_name).to_bytes()));
    let name = String::from_utf8_unchecked(CStr::from_ptr(module.name).to_bytes().to_vec());
    let mut writer = BufWriter::new(ffi_try_result!(
        OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create_new(true)
            .open(file_name),
        -1
    ));

    let module = ptrsx::Module { start: module.start, end: module.end, name };
    let params = ptrsx::Params {
        depth,
        target,
        node,
        offset: (rangel, ranger),
        writer: &mut writer,
    };

    ffi_try_result!(scanner.scanner_with_module(&module, params), -1);

    0
}
