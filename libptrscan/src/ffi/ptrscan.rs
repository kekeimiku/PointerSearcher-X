use core::{
    ffi::{c_char, c_int},
    slice,
};
use std::{
    ffi::{CStr, CString, NulError},
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
}

impl FFIPointerScan {
    fn new() -> Self {
        Self {
            process: None,
            pointer_map: None,
            modules: None,
            modules_ptr: None,
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

/// 附加到进程
#[no_mangle]
pub unsafe extern "C" fn ptrscan_attach_process(ptr: *mut FFIPointerScan, pid: i32) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_result!(Process::attach(pid));
    ptrscan.process = Some(process);

    SUCCESS
}

/// 获取可以作为静态基址的模块列表
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

/// 在内存中创建指针数据
/// 它是根据传入的基本模块地址范围 `module.start` 以及 `module.end` 创建的。
/// `module.pathname` 是一个文件路径，对于库使用者，你应该根据需要处理这个
/// `module.pathname`, 为了方便库使用者自己解析静态地址，规则是使用者自己订的。
/// 例如只传入文件名而不是整个路径，使用索引处理相同的模块名，
/// 扫描指针链会程序根据 `module.name` 输出静态基址部分的内容。
/// 如果你很懂内存，那也可以根据需要传入特定的地址范围，
/// 例如合并相同模块名的连续区域
#[no_mangle]
pub unsafe extern "C" fn ptrscan_create_pointer_map(
    ptr: *mut FFIPointerScan,
    modules: *const FFIModule,
    size: usize,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());

    let unknown_maps = try_result!(process.list_unknown_maps());
    let module_maps = slice::from_raw_parts(modules, size)
        .iter()
        .map(|&FFIModule { start, end, pathname }| {
            let pathname = CStr::from_ptr(pathname).to_str()?.to_string();
            Ok((start..end, pathname))
        })
        .collect::<Result<_, Utf8Error>>();
    let module_maps = try_result!(module_maps);

    let pointer_map = try_result!(process.create_pointer_map(module_maps, unknown_maps));
    ptrscan.pointer_map = Some(pointer_map);

    SUCCESS
}

/// 在文件中创建指针映射
/// 它是根据传入的基本模块地址范围 `module.start` 以及 `module.end` 创建的。
/// `module.pathname` 是一个文件路径，对于库使用者，你应该根据需要处理这个
/// `module.pathname`, 为了方便库使用者自己解析静态地址，规则是使用者自己订的。
/// 例如只传入文件名而不是整个路径，使用索引处理相同的模块名，
/// 扫描指针链会程序根据 `module.name` 输出静态基址部分的内容。
/// 如果你很懂内存，那也可以根据需要传入特定的地址范围，
/// 例如合并相同模块名的连续区域
#[no_mangle]
pub unsafe extern "C" fn ptrscan_create_pointer_map_file(
    ptr: *mut FFIPointerScan,
    modules: *const FFIModule,
    size: usize,
    pathname: *const c_char,
) -> c_int {
    let ptrscan = try_null!(ptr.as_mut());
    let process = try_option!(ptrscan.process.as_ref());
    let path = try_result!(CStr::from_ptr(pathname).to_str());

    let unknown_maps = try_result!(process.list_unknown_maps());
    let module_maps = slice::from_raw_parts(modules, size)
        .iter()
        .map(|&FFIModule { start, end, pathname }| {
            let pathname = CStr::from_ptr(pathname).to_str()?.to_string();
            Ok((start..end, pathname))
        })
        .collect::<Result<_, Utf8Error>>();
    let module_maps = try_result!(module_maps);

    try_result!(process.create_pointer_map_file(module_maps, unknown_maps, path));

    SUCCESS
}

/// 加载指针映射文件到内存中
#[no_mangle]
pub unsafe extern "C" fn ptrscan_load_pointer_map_file(
    ptr: *mut FFIPointerScan,
    path: *const c_char,
) -> c_int {
    let path = try_result!(CStr::from_ptr(path).to_str());
    let ptrscan = try_null!(ptr.as_mut());
    let pointer_map = try_result!(load_pointer_map_file(path));
    ptrscan.pointer_map = Some(pointer_map);

    SUCCESS
}

/// 扫描指针链
/// 它是线程安全的，如果你有多个目标地址参数，可以多线程中同时扫描
/// 关于指针链格式解析，每条以 `$module.name+$offset`
/// 作为静态基址，后续为指针链偏移，以 `.` 分隔，基址 `offset` 和后续偏移都是 10
/// 进制数字
#[no_mangle]
pub unsafe extern "C" fn ptrscan_scan_pointer_chain(
    ptr: *mut FFIPointerScan,
    param: FFIParam,
    pathname: *const c_char,
) -> c_int {
    let FFIParam { addr, depth, srange, lrange, node, last, max, .. } = param;

    let range = (srange.left, srange.right);
    let lrange = lrange.as_ref().copied().map(|r| (r.left, r.right));
    let node = node.as_ref().copied();
    let last = last.as_ref().copied();
    let max = max.as_ref().copied();
    let pathname = try_result!(CStr::from_ptr(try_null!(pathname.as_ref())).to_str());

    let ptrscan = try_null!(ptr.as_ref());
    let pointer_map = try_option!(ptrscan.pointer_map.as_ref());

    #[rustfmt::skip]
    let param = UserParam {
        param: Param { depth, addr, srange: range, lrange },
        node, last, max,
    };

    try_result!(pointer_chain_scan(pointer_map, pathname, param));

    SUCCESS
}

/// 读取内存
/// 内部维护了进程句柄，使用这个库中的读取内存函数可以直接复用内部进程句柄，
/// 当然你自己重新创建一个进程句柄不用这个函数也没什么问题
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
