use core::ffi;
use std::ffi::CString;

use ptrsx_scanner::cmd::SubCommandScan;

#[repr(C)]
pub struct ScannerArgs {}

impl TryInto<ptrsx_scanner::cmd::SubCommandScan> for ScannerArgs {
    type Error = String;

    fn try_into(self) -> Result<ptrsx_scanner::cmd::SubCommandScan, Self::Error> {
        Ok(SubCommandScan {
            file: todo!(),
            target: todo!(),
            depth: todo!(),
            offset: todo!(),
            out: todo!(),
        })
    }
}

#[repr(C)]
pub struct Addr {
    pub start: *const ffi::c_void,
    pub end: *const ffi::c_void,
    // references to ptrsx.map
    // SAFETY: DO NOT use after ptrsx.free()! thus results use-after-free
    pub path: *const ffi::c_char,
}

impl Drop for Addr {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.path as _);
        }
    }
}
