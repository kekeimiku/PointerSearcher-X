use core::slice;
use std::{
    ffi,
    ffi::{CStr, OsStr},
    mem::take,
    os::unix::prelude::OsStrExt,
    path::PathBuf,
    ptr::{self, NonNull},
};

use ptrsx::scanner::PtrsXScanner;
use ptrsx_scanner::cmd::{Offset, SubCommandScan, Target};

use super::error::set_last_error;
use crate::{
    error::set_last_boxed_error,
    ffi_types::{rsmap_to_ffimap, FFIMap},
};

#[no_mangle]
pub unsafe extern "C" fn ptrsx_scanner_init(path: *const ffi::c_char) -> *mut PtrsXScanner {
    if path.is_null() {
        return ptr::null_mut();
    }

    let path = {
        let b = CStr::from_ptr(path).to_bytes();
        OsStr::from_bytes(b)
    };

    match PtrsXScanner::init(path) {
        Ok(p) => Box::into_raw(Box::new(p)),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_scanner_free(ptr: *mut PtrsXScanner) {
    if ptr.is_null() {
        return;
    }

    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn ptrsx_get_select_page(ptr: *mut PtrsXScanner, len: *mut ffi::c_uint) -> *mut FFIMap {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    let ptrsx = { &mut *ptr };
    let mut ffimap = ptrsx.map().iter().map(|m| rsmap_to_ffimap(m)).collect::<Vec<_>>();
    len.write(ffimap.len() as _);
    let ptr = ffimap.as_mut_ptr();
    std::mem::forget(ffimap);
    ptr as _
}

// /// name: file name prefix, NULL-terminated C string; ignored when out is not
// /// null selected_regions: borrowed array of memory regions to scan
// /// regions_len: length for the array above
// /// output_file: borrowed valid relative or absolute output path, pass NULL
// to ///     use default path `${name}.scandata`; NULL-terminated C string
// ///
// /// for other arguments, check documents of
// /// `ptrsx_scanner::cmd::SubCommandScan::perform`
// ///
// /// Errors:
// ///     -1: ptr or name is NULL
// ///     -2: ptrsx did not load a pointer map, or those map is already
// consumed ///     -3: other rust-side errors, check error messages.
// /// SAFETY: Addr.path must not modified by C-Side
#[no_mangle]
pub unsafe extern "C" fn ptrsx_scan_pointer_path(
    ptr: *mut PtrsXScanner,
    name: *const ffi::c_char,
    selected_regions: *const FFIMap,
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

    let pmap = take(&mut ptrsx.bmap);
    let mut mmap = take(&mut ptrsx.map);

    let name = OsStr::from_bytes(CStr::from_ptr(name).to_bytes());

    let selected_regions = slice::from_raw_parts(selected_regions, regions_len as _);
    mmap.retain(|m| selected_regions.iter().any(|FFIMap { start, .. }| start == &m.start));

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
