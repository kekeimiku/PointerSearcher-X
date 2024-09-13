use core::{
    ffi::{c_char, c_int, CStr},
    slice,
};
use std::{
    ffi::{CString, NulError},
    fs::File,
    str::Utf8Error,
};

use super::{
    error::*,
    ffi_type::*,
    scan::{pointer_chain_scan, UserParam},
};
use crate::{
    dump::{load_pointer_map_file, PointerMap, Process, RangeMap},
    scan::Param,
};

pub struct FFIPointerScan {
    process: Option<Process>,
    pointer_map: Option<PointerMap>,
    modules: Option<RangeMap<usize, CString>>,
    modules_ptr: Option<Vec<FFIModule>>,
    set_modules: Option<RangeMap<usize, String>>,
    bitness: Option<u32>,
}

impl FFIPointerScan {
    fn new() -> Self {
        Self {
            process: None,
            pointer_map: None,
            modules: None,
            modules_ptr: None,
            set_modules: None,
            bitness: None,
        }
    }
}

/// 初始化
#[no_mangle]
pub unsafe extern "C" fn ptrscan_init() -> *mut FFIPointerScan {
    let ptrscan = FFIPointerScan::new();
    Box::into_raw(Box::new(ptrscan))
}

/// 释放
#[no_mangle]
pub unsafe extern "C" fn ptrscan_free(ptr: *mut FFIPointerScan) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// 获取版本号
#[no_mangle]
pub const unsafe extern "C" fn ptrscan_version() -> *const c_char {
    let bytes = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    CStr::from_bytes_with_nul_unchecked(bytes).as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_set_process(ptr: *mut FFIPointerScan, pid: i32) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_result!(Process::attach(pid));
    ptrscan.process = Some(process);

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_list_modules(
    ptr: *mut FFIPointerScan,
    modules: *mut *const FFIModule,
    size: *mut usize,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());

    let ffi_modules = try_result!(process.list_image_maps())
        .into_iter()
        .map(|(range, pathname)| {
            let pathname = CString::new(pathname)?;
            Ok((range, pathname))
        })
        .collect::<Result<_, NulError>>();
    let ffi_modules = try_result!(ffi_modules);
    ptrscan.modules = Some(ffi_modules);

    let ffi_modules_ptr = ptrscan
        .modules
        .as_ref()
        .unwrap_unchecked()
        .iter()
        .map(|(range, pathname)| FFIModule {
            start: range.start,
            end: range.end,
            pathname: pathname.as_ptr(),
        })
        .collect();
    ptrscan.modules_ptr = Some(ffi_modules_ptr);

    let ffi_modules_ptr = ptrscan.modules_ptr.as_ref().unwrap_unchecked();
    size.write(ffi_modules_ptr.len());
    modules.write(ffi_modules_ptr.as_ptr());

    SUCCESS
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn ptrscan_list_modules_pince(
    ptr: *mut FFIPointerScan,
    modules: *mut *const FFIModule,
    size: *mut usize,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());

    let ffi_modules = try_result!(process.list_image_maps_pince())
        .into_iter()
        .map(|(range, pathname)| {
            let pathname = CString::new(pathname)?;
            Ok((range, pathname))
        })
        .collect::<Result<_, NulError>>();
    let ffi_modules = try_result!(ffi_modules);
    ptrscan.modules = Some(ffi_modules);

    let ffi_modules_ptr = ptrscan
        .modules
        .as_ref()
        .unwrap_unchecked()
        .iter()
        .map(|(range, pathname)| FFIModule {
            start: range.start,
            end: range.end,
            pathname: pathname.as_ptr(),
        })
        .collect();
    ptrscan.modules_ptr = Some(ffi_modules_ptr);

    let ffi_modules_ptr = ptrscan.modules_ptr.as_ref().unwrap_unchecked();
    size.write(ffi_modules_ptr.len());
    modules.write(ffi_modules_ptr.as_ptr());

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_set_pointer_offset_symbol(
    ptr: *mut FFIPointerScan,
    symbol: *const c_char,
) -> c_int {
    let _ptrscan = try_null!(ptr.as_mut());
    let _symbol = try_result!(CStr::from_ptr(symbol).to_str());

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_set_base_offset_symbol(
    ptr: *mut FFIPointerScan,
    symbol: *const c_char,
) -> c_int {
    let _ptrscan = try_null!(ptr.as_mut());
    let _symbol = try_result!(CStr::from_ptr(symbol).to_str());

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_set_bitness(ptr: *mut FFIPointerScan, bitness: u32) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    match bitness {
        4 | 8 => {
            ptrscan.bitness = Some(bitness);
            SUCCESS
        }
        _ => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_set_modules(
    ptr: *mut FFIPointerScan,
    modules: *const FFIModule,
    size: usize,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());

    let module_maps = slice::from_raw_parts(modules, size)
        .iter()
        .map(|&FFIModule { start, end, pathname }| {
            let pathname = CStr::from_ptr(pathname).to_str()?.to_string();
            Ok((start..end, pathname))
        })
        .collect::<Result<_, Utf8Error>>();
    let module_maps = try_result!(module_maps);

    ptrscan.set_modules = Some(module_maps);

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_create_pointer_map(ptr: *mut FFIPointerScan) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());

    let module_maps = try_option!(&ptrscan.set_modules).clone();
    let unknown_maps = try_result!(process.list_unknown_maps());

    let bitness = try_option!(ptrscan.bitness);

    if bitness == 4 {
        let pointer_map = try_result!(process.create_pointer_map_4(module_maps, unknown_maps));
        ptrscan.pointer_map = Some(pointer_map);
    } else if bitness == 8 {
        let pointer_map = try_result!(process.create_pointer_map_8(module_maps, unknown_maps));
        ptrscan.pointer_map = Some(pointer_map);
    } else {
        set_last_error("invalid bitness");
        return -1;
    }

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_create_pointer_map_file(
    ptr: *mut FFIPointerScan,
    pathname: *const c_char,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());
    let path = try_result!(CStr::from_ptr(pathname).to_str());
    let module_maps = try_option!(&ptrscan.set_modules).clone();
    let unknown_maps = try_result!(process.list_unknown_maps());

    let bitness = try_option!(ptrscan.bitness);

    if bitness == 4 {
        try_result!(process.create_pointer_map_file_4(module_maps, unknown_maps, path));
    } else if bitness == 8 {
        try_result!(process.create_pointer_map_file_8(module_maps, unknown_maps, path));
    } else {
        set_last_error("invalid bitness");
        return -1;
    }

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_load_pointer_map_file(
    ptr: *mut FFIPointerScan,
    pathname: *const c_char,
) -> c_int {
    let path = try_result!(CStr::from_ptr(pathname).to_str());
    let ptrscan = try_null!(ptr.as_mut());
    let pointer_map = try_result!(load_pointer_map_file(path));
    ptrscan.pointer_map = Some(pointer_map);

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_scan_pointer_chain(
    ptr: *mut FFIPointerScan,
    param: FFIParam,
    pathname: *const c_char,
) -> c_int {
    let FFIParam { addr, depth, srange, lrange, node, last, max, cycle, .. } = param;

    let srange = srange.left..srange.right;
    let lrange = lrange.as_ref().copied().map(|r| (r.left..r.right));
    let node = node.as_ref().copied();
    let last = last.as_ref().copied();
    let max = max.as_ref().copied();

    #[rustfmt::skip]
    let param = UserParam {
        param: Param { depth, addr, srange, lrange },
        node, last, max, cycle
    };

    let ptrscan = try_null!(ptr.as_ref());
    let pointer_map = try_option!(ptrscan.pointer_map.as_ref());

    if pathname.is_null() {
        let stdout = std::io::stdout();
        try_result!(pointer_chain_scan(pointer_map, stdout, param));
    } else {
        let pathname = try_result!(CStr::from_ptr(try_null!(pathname.as_ref())).to_str());
        let file = try_result!(File::options().append(true).create_new(true).open(pathname));
        try_result!(pointer_chain_scan(pointer_map, file, param));
    }

    SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ptrscan_read_memory_exact(
    ptr: *mut FFIPointerScan,
    addr: usize,
    data: *mut u8,
    size: usize,
) -> c_int {
    let ptrscan = try_null!(ptr.as_ref());
    let process = try_option!(ptrscan.process.as_ref());
    let data = slice::from_raw_parts_mut(data, size);
    try_result!(process.read_memory_exact(addr, data));

    SUCCESS
}
