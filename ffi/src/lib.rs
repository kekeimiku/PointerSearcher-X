#![allow(clippy::missing_safety_doc)]

use core::{
    cell::RefCell,
    ffi::{c_char, c_int, CStr},
    mem, ptr,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use ptrsx::{Module, PtrsxScanner, UserParam};
use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) }
}

fn set_last_error(err: impl ToString) {
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(unsafe { CString::from_vec_unchecked(err.to_string().into_bytes()) });
    });
}

#[repr(C)]
pub struct Param {
    pub addr: usize,
    pub depth: usize,
    pub left: usize,
    pub right: usize,
    pub use_module: bool,
    pub use_cycle: bool,
    pub node: Option<ptr::NonNull<usize>>,
    pub max: Option<ptr::NonNull<usize>>,
    pub last: Option<ptr::NonNull<isize>>,
}

const PTR_NULL: &str = "ptr is null";
const NO_OPEN_PROCESS: &str = "no process is opened";
const CACHE_INVALID: &str = "modules cache invalid";
const PTR_CHAIN_INVALID: &str = "pointer chain invalid";

macro_rules! null_ptr {
    ($m:expr) => {
        match $m {
            Some(val) => val,
            None => {
                set_last_error(PTR_NULL);
                return -1;
            }
        }
    };
}

macro_rules! ref_proc {
    ($m:expr) => {
        match $m {
            Some(val) => val,
            None => {
                set_last_error(NO_OPEN_PROCESS);
                return -2;
            }
        }
    };
}

macro_rules! ref_index {
    ($m:expr) => {
        match $m {
            Some(val) => val,
            None => {
                set_last_error(CACHE_INVALID);
                return -3;
            }
        }
    };
}

macro_rules! error {
    ($m:expr) => {
        match $m {
            Ok(val) => val,
            Err(err) => {
                set_last_error(err);
                return -4;
            }
        }
    };
}

#[derive(Default)]
pub struct PointerScanTool {
    scan: PtrsxScanner,
    proc: Option<Process>,
    index: Option<HashMap<String, usize>>,
}

#[no_mangle]
pub extern "C" fn ptrs_init() -> *mut PointerScanTool {
    Box::into_raw(Box::default())
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_free(ptr: *mut PointerScanTool) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn get_last_error() -> *const c_char {
    LAST_ERROR.with(|prev| match prev.borrow().as_ref() {
        Some(err) => err.as_ptr(),
        None => ptr::null_mut(),
    })
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_set_proc(ptr: *mut PointerScanTool, pid: Pid) -> c_int {
    let proc = error!(Process::open(pid));
    let this = null_ptr!(ptr.as_mut());
    this.proc = Some(proc);
    0
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_create_pointer_map(
    ptr: *mut PointerScanTool,
    info_path: *const c_char,
    bin_path: *const c_char,
) -> c_int {
    let info_file = error!(CStr::from_ptr(null_ptr!(info_path.as_ref())).to_str());
    let bin_file = error!(CStr::from_ptr(null_ptr!(bin_path.as_ref())).to_str());

    dbg!(info_file, bin_file);

    let this = null_ptr!(ptr.as_ref());
    let proc = ref_proc!(this.proc.as_ref());
    error!(this.scan.create_pointer_map(proc, info_file, bin_file));

    0
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_load_pointer_map(
    ptr: *mut PointerScanTool,
    info_path: *const c_char,
    bin_path: *const c_char,
) -> c_int {
    let scan = &mut null_ptr!(ptr.as_mut()).scan;
    let info_path = error!(CStr::from_ptr(null_ptr!(info_path.as_ref())).to_str());
    dbg!(info_path);
    let file = error!(File::open(info_path));
    error!(scan.load_modules_info(file));

    let bin_path = error!(CStr::from_ptr(null_ptr!(bin_path.as_ref())).to_str());
    dbg!(bin_path);

    let file = error!(File::open(bin_path));
    error!(scan.load_pointer_map(file));
    0
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_scan_pointer_chain(
    ptr: *mut PointerScanTool,
    param: Param,
    file_path: *const c_char,
) -> c_int {
    let scan = &null_ptr!(ptr.as_ref()).scan;

    let file_name = error!(CStr::from_ptr(null_ptr!(file_path.as_ref())).to_str());

    let Param { addr, depth, left, right, use_module, use_cycle, node, max, last } = param;
    let node = node.map(|x| x.as_ref()).copied();
    let max = max.map(|x| x.as_ref()).copied();
    let last = last.map(|x| x.as_ref()).copied();

    dbg!(depth, addr, left, right, use_module, node, max, last);

    let range = (left, right);
    let param = UserParam { depth, addr, range, use_module, use_cycle, node, max, last };

    error!(scan.pointer_chain_scanner(param, file_name));

    0
}

#[no_mangle]
pub unsafe extern "C" fn compare_two_file(file1: *const c_char, file2: *const c_char, outfile: *const c_char) -> c_int {
    use std::fs;

    let file1 = error!(CStr::from_ptr(null_ptr!(file1.as_ref())).to_str());
    let file2 = error!(CStr::from_ptr(null_ptr!(file2.as_ref())).to_str());
    let outfile = error!(CStr::from_ptr(null_ptr!(outfile.as_ref())).to_str());

    dbg!(file1, file2, outfile);

    let b1 = error!(fs::read_to_string(file1));
    let b2 = error!(fs::read_to_string(file2));
    let s1 = b1.lines().collect::<HashSet<_>>();
    let s2 = b2.lines().collect::<HashSet<_>>();

    let f = error!(File::options().append(true).create(true).open(outfile));
    let mut w = BufWriter::new(f);
    error!(s1.intersection(&s2).try_for_each(|s| writeln!(w, "{s}")));

    0
}

fn get_pointer_chain_address<P>(proc: &P, index: &HashMap<String, usize>, chain: &str) -> Option<usize>
where
    P: VirtualMemoryRead + ProcessInfo,
{
    let mut parts = chain.trim_end().split(['+', '@']);
    let module_name = parts.next()?;
    let offset = parts.next_back()?.parse().ok()?;
    let items = parts.map(|s| s.parse());

    let mut address = index.get(module_name).copied()?;

    let mut buf = [0; mem::size_of::<usize>()];
    for item in items {
        let item = item.ok()?;
        let offset = address.checked_add_signed(item)?;
        proc.read_exact_at(&mut buf, offset).ok()?;
        address = usize::from_le_bytes(buf);
    }
    address.checked_add_signed(offset)
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_get_chain_addr(
    ptr: *mut PointerScanTool,
    chain: *const c_char,
    addr: *mut usize,
) -> c_int {
    let ptr = null_ptr!(ptr.as_ref());
    let proc = ref_proc!(ptr.proc.as_ref());
    let index = ref_index!(ptr.index.as_ref());
    let chain = error!(CStr::from_ptr(null_ptr!(chain.as_ref())).to_str());

    dbg!(chain);

    match get_pointer_chain_address(proc, index, chain) {
        Some(ad) => {
            addr.write(ad);
            0
        }
        None => {
            set_last_error(PTR_CHAIN_INVALID);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_filter_invalid(
    ptr: *mut PointerScanTool,
    infile: *const c_char,
    outfile: *const c_char,
) -> c_int {
    let ptr = null_ptr!(ptr.as_ref());
    let proc = ref_proc!(ptr.proc.as_ref());
    let index = ref_index!(ptr.index.as_ref());

    let infile = error!(CStr::from_ptr(null_ptr!(infile.as_ref())).to_str());
    let outfile = error!(CStr::from_ptr(null_ptr!(outfile.as_ref())).to_str());

    dbg!(infile, outfile);

    let infile = error!(File::open(infile));
    let mut reader = BufReader::with_capacity(0x80000, infile);
    let line_buf = &mut String::with_capacity(0x2000);

    let outfile = error!(File::options().append(true).create_new(true).open(outfile));
    let mut writer = BufWriter::with_capacity(0x80000, outfile);

    loop {
        let size = error!(reader.read_line(line_buf));
        if size == 0 {
            break;
        }
        if get_pointer_chain_address(proc, index, line_buf.trim()).is_some() {
            error!(writer.write_all(line_buf.as_bytes()))
        }
        line_buf.clear()
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_filter_value(
    ptr: *mut PointerScanTool,
    infile: *const c_char,
    outfile: *const c_char,
    data: *const u8,
    size: usize,
) -> c_int {
    let ptr = null_ptr!(ptr.as_ref());
    let proc = ref_proc!(ptr.proc.as_ref());
    let index = ref_index!(ptr.index.as_ref());

    let value = null_ptr!(ptr::slice_from_raw_parts(data, size).as_ref());
    let infile = error!(CStr::from_ptr(null_ptr!(infile.as_ref())).to_str());
    let outfile = error!(CStr::from_ptr(null_ptr!(outfile.as_ref())).to_str());

    dbg!(infile, outfile, value);

    let infile = error!(File::open(infile));
    let mut reader = BufReader::with_capacity(0x80000, infile);
    let line_buf = &mut String::with_capacity(0x2000);

    let outfile = error!(File::options().append(true).create_new(true).open(outfile));
    let mut writer = BufWriter::with_capacity(0x80000, outfile);

    let mut value_buf = vec![0_u8; value.len()];

    loop {
        let size = error!(reader.read_line(line_buf));
        if size == 0 {
            break;
        }

        if get_pointer_chain_address(proc, index, line_buf.trim())
            .and_then(|addr| proc.read_exact_at(&mut value_buf, addr).ok())
            .is_some_and(|_| value_buf == value)
        {
            error!(writer.write_all(line_buf.as_bytes()))
        }

        line_buf.clear()
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn refresh_modules_cache(ptr: *mut PointerScanTool) -> c_int {
    let ptr = null_ptr!(ptr.as_mut());
    let proc = ref_proc!(ptr.proc.as_ref());

    let mappings = proc
        .get_maps()
        .flatten()
        .filter(|x| x.is_write() && x.is_read())
        .collect::<Vec<_>>();
    let modules = mappings
        .iter()
        .flat_map(|x| Some(Module { start: x.start(), end: x.end(), name: x.name()? }))
        .fold(Vec::<Module>::with_capacity(mappings.len()), |mut acc, cur| {
            match acc.last_mut() {
                Some(last) if last.name == cur.name => last.end = cur.end,
                _ => acc.push(cur),
            }
            acc
        })
        .into_iter()
        .map(|Module { start, name, .. }| {
            let name = Path::new(name).file_name().and_then(|s| s.to_str()).unwrap_or(name);
            (name.to_string(), start)
        })
        .collect::<HashMap<String, usize>>();

    ptr.index = Some(modules);

    0
}

#[no_mangle]
pub unsafe extern "C" fn ptrs_filter_addr(
    ptr: *mut PointerScanTool,
    infile: *const c_char,
    outfile: *const c_char,
    addr: usize,
) -> c_int {
    let ptr = null_ptr!(ptr.as_ref());
    let proc = ref_proc!(ptr.proc.as_ref());
    let index = ref_index!(ptr.index.as_ref());
    let infile = error!(CStr::from_ptr(null_ptr!(infile.as_ref())).to_str());
    let outfile = error!(CStr::from_ptr(null_ptr!(outfile.as_ref())).to_str());

    dbg!(infile, outfile, addr);

    let infile = error!(File::open(infile));
    let mut reader = BufReader::with_capacity(0x80000, infile);
    let line_buf = &mut String::with_capacity(0x2000);

    let outfile = error!(File::options().append(true).create_new(true).open(outfile));
    let mut writer = BufWriter::with_capacity(0x80000, outfile);

    loop {
        let size = error!(reader.read_line(line_buf));
        if size == 0 {
            break;
        }

        if get_pointer_chain_address(proc, index, line_buf.trim()).is_some_and(|x| x == addr) {
            error!(writer.write_all(line_buf.as_bytes()));
        }

        line_buf.clear()
    }

    0
}
