#![allow(clippy::missing_safety_doc)]

pub mod error;

use std::{
    ffi,
    ffi::{CStr, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::ffi::OsStrExt,
    ptr,
};

use error::set_last_error;
use ptrsx::c::create_pointer_map_helper;
use vmmap::{Pid, Process};

pub struct PtrsX {
    pub proc: Process,
}

impl PtrsX {
    pub fn init(pid: Pid) -> Result<PtrsX, vmmap::Error> {
        let proc = Process::open(pid)?;
        Ok(Self { proc })
    }
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_init(pid: ffi::c_int) -> *mut PtrsX {
    let ptrsx = match PtrsX::init(pid) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(ptrsx))
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_free(ptr: *mut PtrsX) {
    if ptr.is_null() {
        return;
    }

    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_create_pointer_map(ptr: *mut PtrsX, path: *const ffi::c_char) -> ffi::c_int {
    let ptrsx = {
        if path.is_null() {
            return -1;
        }

        &mut *ptr
    };

    let path = {
        if path.is_null() {
            return -1;
        }
        let b = CStr::from_ptr(path).to_bytes();
        OsStr::from_bytes(b)
    };

    let proc = &ptrsx.proc;

    let file = OpenOptions::new().write(true).append(true).create_new(true).open(path);
    let mut out = match file {
        Ok(f) => BufWriter::new(f),
        Err(e) => {
            set_last_error(e);
            return -2;
        }
    };

    match create_pointer_map_helper(proc, &mut out) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(e);
            -2
        }
    }
}
