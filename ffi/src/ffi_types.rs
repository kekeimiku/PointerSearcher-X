use core::ffi;
use std::{ffi::CString, os::unix::prelude::OsStrExt};

use ptrsx::map::Map;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FFIMap {
    pub start: usize,
    pub end: usize,
    // references to a pointer created by [`CString::into_raw`](https://stdrs.dev/nightly/x86_64-unknown-linux-gnu/std/ffi/struct.CString.html#method.into_raw)
    // SAFETY: DO NOT use after ptrsx.free()! thus results use-after-free
    pub path: *const ffi::c_char,
}

impl Drop for FFIMap {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.path as _);
        }
    }
}

pub unsafe fn rsmap_to_ffimap(map: &Map) -> FFIMap {
    let path = CString::from(
        map.path
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|&c| std::num::NonZeroU8::new_unchecked(c))
            .collect::<Vec<_>>(),
    );

    println!("{:?}", path);

    let path = path.into_raw();

    FFIMap { start: map.start, end: map.end, path }
}
