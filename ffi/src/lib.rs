#![allow(clippy::missing_safety_doc)]

pub mod error;
mod ffi_types;

use std::{
    ffi,
    ffi::{CStr, CString, OsStr},
    fs::{File, OpenOptions},
    io::BufWriter,
    os::unix::ffi::OsStrExt,
    ptr,
    sync::Arc, collections::BTreeMap,
};

use dumper::map::Map;
use error::set_last_error;
use ptrsx::c::create_pointer_map_helper;
use ptrsx_scanner::b::load_pointer_map;
use utils::consts::Address;
use vmmap::{Pid, Process};

use crate::ffi_types::Addr;

pub struct PtrsX {
    pub proc: Process<Arc<File>>,
    pub map: Option<Vec<dumper::map::Map>>,
    bmap: Option<BTreeMap<Address, Address>>,
    addr_vec: Option<Vec<ffi_types::Addr>>,
}

impl PtrsX {
    pub fn init(pid: Pid) -> Result<PtrsX, vmmap::Error> {
        let proc = Process::open(pid)?;
        Ok(Self { proc, map: None, addr_vec: None, bmap: None })
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
    if ptr.is_null() || path.is_null() {
        return -1;
    }

    let ptrsx = { &mut *ptr };

    let path = {
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

// returns read-only rust-owned array if read without error
// returns NULL if any error occured
#[no_mangle]
pub unsafe extern "C" fn ptrsx_load_pointer_map(
    ptr: *mut PtrsX,
    path: *const ffi::c_char,
    length: *mut ffi::c_uint,
) -> *const Addr {
    const C_NULL: usize = 0;
    if ptr.is_null() || path.is_null() {
        return C_NULL as _;
    }

    let ptrsx = { &mut *ptr };

    let path = {
        let b = CStr::from_ptr(path).to_bytes();
        OsStr::from_bytes(b)
    };

    match load_pointer_map(path) {
        Ok((bmap, map)) => {
            ptrsx.bmap = Some(bmap);

            length.write(map.len() as _);
            ptrsx.addr_vec = Some(
                map.iter()
                    .map(|Map { ref start, end, path }| {
                        let path = CString::from(
                            path.as_os_str()
                                .as_bytes()
                                .iter()
                                // on most modern OS NULL is not allowd in path; may should check
                                .map(|&c| std::num::NonZeroU8::new_unchecked(c))
                                .collect::<Vec<_>>(),
                        )
                        .into_raw();
                        Addr { start: *start as _, end: *end as _, path }
                    })
                    .collect::<Vec<_>>(),
            );
            ptrsx.map = Some(map);
            ptrsx.addr_vec.as_ref().unwrap().as_ptr()
        }
        Err(e) => {
            set_last_error(e);
            return C_NULL as _;
        }
    }
}
