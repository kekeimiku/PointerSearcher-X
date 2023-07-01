use std::{
    ffi::{self, CStr, OsStr},
    os::unix::prelude::OsStrExt,
    path::Path,
    pin::Pin,
    ptr,
};

use ptrsx::sc64::PtrsxScanner;

use crate::{ffi_try_result, ffi_types::FFIPAGE};

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
    let mut pages = ffi_try_result![
        (*ptr)
            .0
            .pages()
            .iter()
            .map(FFIPAGE::try_from)
            .collect::<Result<Vec<_>, _>>(),
        ptr::null_mut()
    ];

    let ptr = pages.as_mut_ptr();
    std::mem::forget(pages);

    ptr
}
