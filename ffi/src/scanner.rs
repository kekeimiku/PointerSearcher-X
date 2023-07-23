use std::{
    ffi::{self, CStr, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    path::Path,
    pin::Pin,
    ptr::{self, slice_from_raw_parts},
};

use ptrsx::{c64::Page, s64::Params, sc64::PtrsxScanner};

use crate::{
    error::StrErrorWrap,
    ffi_try_result,
    ffi_types::{FFIParams, FFIPAGE},
};

pub struct Scanner<'a>(Pin<Box<PtrsxScanner<'a>>>);

#[no_mangle]
pub unsafe extern "C" fn scanner_init<'a>(in_file: *const ffi::c_char) -> *mut Scanner<'a> {
    let in_file = Path::new(OsStr::from_bytes(CStr::from_ptr(in_file).to_bytes()));
    let scanner = ffi_try_result![PtrsxScanner::new(in_file), ptr::null_mut()];
    Box::into_raw(Box::new(Scanner(scanner)))
}

#[no_mangle]
pub unsafe extern "C" fn scanner_free(ptr: *mut Scanner) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn scanner_get_pages_len(ptr: *mut Scanner) -> ffi::c_int {
    (*ptr).0.pages().len() as ffi::c_int
}

#[no_mangle]
pub unsafe extern "C" fn scanner_get_pages(ptr: *mut Scanner) -> *mut FFIPAGE {
    let mut pages = (*ptr).0.pages().iter().map(FFIPAGE::from).collect::<Vec<_>>();
    let ptr = pages.as_mut_ptr();
    std::mem::forget(pages);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn scanner_pointer_chain(
    ptr: *mut Scanner,
    pages: *const FFIPAGE,
    len: ffi::c_ulonglong,
    params: FFIParams,
) -> ffi::c_int {
    let ptrsx = &(*ptr).0;
    let ffi_map = &*slice_from_raw_parts(pages, len as _);
    let pages = ffi_map.iter().map(Page::from);
    let rev_map = ptrsx.get_rev_pointer_map();
    let dir = Path::new(OsStr::from_bytes(CStr::from_ptr(params.out_dir).to_bytes()));
    for page in pages {
        let points = ptrsx.range_address(&page).collect::<Vec<_>>();
        let name = Path::new(page.path)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or(StrErrorWrap("get region name error"));
        let name = ffi_try_result![name, -1];
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(true)
            .open(dir.join(name).with_extension("scandata"));
        let file = ffi_try_result![file, -1];
        let params = Params {
            base: page.start as usize,
            depth: params.depth as usize,
            range: (params.rangel as usize, params.ranger as usize),
            points: &points,
            target: params.target as usize,
            writer: &mut BufWriter::new(file),
        };
        ffi_try_result![ptrsx.scan(&rev_map, params), -1]
    }

    0
}
