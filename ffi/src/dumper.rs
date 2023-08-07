use std::{
    ffi::{self, CStr, OsStr},
    fs::OpenOptions,
    io::BufWriter,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use ptrsx::default_dump_ptr;
use vmmap::Process;

use super::ffi_try_result;

#[no_mangle]
pub unsafe extern "C" fn ptrsx_dumper_init(pid: ffi::c_int, out_file: *const ffi::c_char) -> ffi::c_int {
    let out_file = Path::new(OsStr::from_bytes(CStr::from_ptr(out_file).to_bytes()));
    let proc = ffi_try_result![Process::open(pid), -1];
    let file = ffi_try_result![OpenOptions::new().create_new(true).write(true).open(out_file), -1];
    let mut writer = BufWriter::new(file);
    ffi_try_result![default_dump_ptr(&proc, &mut writer), -1];
    0
}
