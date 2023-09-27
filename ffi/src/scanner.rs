use std::{
    ffi::{c_char, c_int, CStr, CString, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    path::Path,
    ptr::slice_from_raw_parts,
};

use ptrsx::PtrsxScanner;

use super::{ffi_try_result, Page, PageVec, Params};

pub struct Scanner<'a> {
    engine: PtrsxScanner<'a>,
    pages: Vec<Page>,
}

#[no_mangle]
pub unsafe extern "C" fn scanner_init_with_file(path: *const c_char, ptr: *mut *mut Scanner) -> c_int {
    let file = Path::new(OsStr::from_bytes(CStr::from_ptr(path).to_bytes()));
    let engine = ffi_try_result![PtrsxScanner::load_with_file(file), -1];
    let pages = engine
        .pages()
        .iter()
        .map(|x| Page {
            start: x.start,
            end: x.end,
            path: CString::from_vec_unchecked(x.path.into()).into_raw(),
        })
        .collect::<Vec<_>>();
    *ptr = Box::into_raw(Box::new(Scanner { engine, pages }));
    0
}

#[no_mangle]
pub unsafe extern "C" fn scanner_free(ptr: *mut Scanner) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn scanner_get_pages(ptr: *const Scanner) -> PageVec {
    let pages = &(*ptr).pages;
    let len = pages.len();
    let data = pages.as_ptr();
    PageVec { len, data }
}

#[no_mangle]
pub unsafe extern "C" fn scanner_pointer_chain(ptr: *mut Scanner, pages: *const PageVec, params: Params) -> c_int {
    let ptrsx = &(*ptr).engine;
    let pages = &(*pages);
    let ffi_map = &*slice_from_raw_parts(pages.data, pages.len);
    let pages = ffi_map.iter().map(|x| ptrsx::Page {
        start: x.start,
        end: x.end,
        path: std::str::from_utf8_unchecked(CStr::from_ptr(x.path).to_bytes()),
    });
    let dir = Path::new(OsStr::from_bytes(CStr::from_ptr(params.dir).to_bytes()));
    for page in pages {
        let points = ptrsx.range_address(&page).collect::<Vec<_>>();
        let name = Path::new(page.path)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("get region name error");
        let name = ffi_try_result![name, -1];
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(true)
            .open(dir.join(format!("{name}.scandata")));
        let file = ffi_try_result![file, -1];
        let params = ptrsx::Params {
            base: page.start,
            depth: params.depth,
            node: params.node,
            offset: (params.rangel, params.ranger),
            points: &points,
            target: params.target,
            writer: &mut BufWriter::new(file),
        };
        ffi_try_result![ptrsx.scan(params), -1]
    }

    0
}
