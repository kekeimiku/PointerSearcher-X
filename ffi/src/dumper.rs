use std::{
    ffi::{c_char, c_int, CStr, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use ptrsx::default_dump_ptr;
use vmmap::Process;

use super::ffi_try_result;

#[no_mangle]
pub unsafe extern "C" fn dumper_to_file(pid: c_int, path: *const c_char) -> c_int {
    let file = Path::new(OsStr::from_bytes(CStr::from_ptr(path).to_bytes()));
    let proc = ffi_try_result![Process::open(pid), -1];
    let file = ffi_try_result![OpenOptions::new().create_new(true).write(true).open(file), -1];
    let mut writer = BufWriter::new(file);
    ffi_try_result![default_dump_ptr(&proc, &mut writer), -1];
    0
}
