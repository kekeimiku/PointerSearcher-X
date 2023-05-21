#![allow(clippy::missing_safety_doc)]

pub mod error;
mod ffi_types;

use std::{
    collections::BTreeMap,
    ffi,
    ffi::{CStr, CString, OsStr},
    fs::{File, OpenOptions},
    io::BufWriter,
    mem::take,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    ptr::{self, NonNull},
    slice,
    sync::Arc,
};

use dumper::map::Map;
use error::{set_last_boxed_error, set_last_error};
use ptrsx::c::create_pointer_map_helper;
use ptrsx_scanner::{
    b::load_pointer_map,
    cmd::{Offset, SubCommandScan, Target},
};
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
                        // TODO: return path with length to tell users path is not NULL-terminated
                        // and avoid unnessacary copy?
                        let path = CString::from(
                            path.as_os_str()
                                .as_bytes()
                                .iter()
                                // on most modern OS NULL is not allowd in path; may should check
                                .map(|&c| std::num::NonZeroU8::new_unchecked(c))
                                .collect::<Vec<_>>(),
                        )
                        .into_raw();
                        Addr { start: *start, end: *end, path }
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

/// name: file name prefix, NULL-terminated C string; ignored when out is not null
/// selected_regions: borrowed array of memory regions to scan
/// regions_len: length for the array above
/// output_file: borrowed valid relative or absolute output path, pass NULL to
///     use default path `${name}.scandata`; NULL-terminated C string
///
/// for other arguments, check documents of
/// `ptrsx_scanner::cmd::SubCommandScan::perform`
///
/// Errors:
///     -1: ptr or name is NULL
///     -2: ptrsx did not load a pointer map, or those map is already consumed
///     -3: other rust-side errors, check error messages.
/// SAFETY: Addr.path must not modified by C-Side
#[no_mangle]
pub unsafe extern "C" fn ptrsx_scan_pointer_path(
    ptr: *mut PtrsX,
    name: *const ffi::c_char,
    selected_regions: *const Addr,
    regions_len: u32,
    output_file: *const ffi::c_char,
    depth: u32,
    target_addr: usize,
    offset_ahead: usize,
    offset_behind: usize,
) -> ffi::c_int {
    if ptr.is_null() || name.is_null() {
        return -1;
    }

    let ptrsx = { &mut *ptr };

    let Some((pmap, mut mmap)) = take(&mut ptrsx.bmap).zip(take(&mut ptrsx.map)) else {
        return -2;
    };

    let name = OsStr::from_bytes(CStr::from_ptr(name).to_bytes());

    let selected_regions = slice::from_raw_parts(selected_regions, regions_len as _);

    mmap.retain(|m| selected_regions.iter().any(|Addr { start, .. }| start == &m.start));

    let out = NonNull::new(output_file as *mut ffi::c_char)
        .map(|p| PathBuf::from(OsStr::from_bytes(CStr::from_ptr(p.as_ptr() as _).to_bytes())));

    match SubCommandScan::perform(
        name,
        (pmap, mmap),
        Target(target_addr),
        out,
        depth as _,
        Offset((offset_ahead, offset_behind)),
    ) {
        Ok(()) => 0,
        Err(e) => {
            set_last_boxed_error(e);
            -2
        }
    }
}
