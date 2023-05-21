use std::{
    ffi::{self, CStr, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    ptr,
};

use ptrsx::dumper::PtrsXDumper;

use super::error::set_last_error;

#[no_mangle]
pub unsafe extern "C" fn ptrsx_dumper_init(pid: ffi::c_int) -> *mut PtrsXDumper {
    match PtrsXDumper::init(pid) {
        Ok(p) => Box::into_raw(Box::new(p)),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_dumper_free(ptr: *mut PtrsXDumper) {
    if ptr.is_null() {
        return;
    }

    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_create_pointer_map(ptr: *mut PtrsXDumper, path: *const ffi::c_char) -> ffi::c_int {
    if ptr.is_null() || path.is_null() {
        return -1;
    }

    let ptrsx = { &mut *ptr };

    let path = {
        let b = CStr::from_ptr(path).to_bytes();
        OsStr::from_bytes(b)
    };

    let file = OpenOptions::new().write(true).append(true).create_new(true).open(path);
    let mut out = match file {
        Ok(f) => BufWriter::new(f),
        Err(e) => {
            set_last_error(e);
            return -2;
        }
    };

    match ptrsx.create_pointer_map_helper(&mut out) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(e);
            -2
        }
    }
}
