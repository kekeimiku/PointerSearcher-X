use core::slice;
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr,
};

use super::ptrscan_bindgen::*;

pub struct Param {
    pub addr: usize,
    pub depth: usize,
    pub srange: (usize, usize),
    pub lrange: Option<(usize, usize)>,
    pub node: Option<usize>,
    pub last: Option<isize>,
    pub max: Option<usize>,
    pub cycle: bool,
}

#[derive(Debug)]
pub struct Module {
    pub start: usize,
    pub end: usize,
    pub pathname: String,
}

pub struct PointerScan {
    ptr: *mut FFIPointerScan,
}

unsafe impl Sync for PointerScan {}

impl Drop for PointerScan {
    fn drop(&mut self) {
        unsafe { ptrscan_free(self.ptr) }
    }
}

impl Default for PointerScan {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerScan {
    pub fn new() -> Self {
        let ptr = unsafe { ptrscan_init() };
        PointerScan { ptr }
    }

    #[allow(dead_code)]
    pub fn version<'a>() -> &'a str {
        unsafe {
            let ptr = ptrscan_version();
            CStr::from_ptr(ptr).to_str().unwrap_unchecked()
        }
    }

    pub fn attach_process(&mut self, pid: i32) -> Result<(), String> {
        let ret = unsafe { ptrscan_attach_process(self.ptr, pid) };
        if ret != SUCCESS {
            let err = unsafe {
                let ptr = get_last_error(ret);
                CStr::from_ptr(ptr).to_str().unwrap_unchecked()
            }
            .to_string();
            return Err(err);
        }
        Ok(())
    }

    pub fn list_modules(&self) -> Result<Vec<Module>, String> {
        let mut modules_ptr = ptr::null();
        let mut size = 0;
        let ret = unsafe { ptrscan_list_modules(self.ptr, &mut modules_ptr, &mut size) };
        if ret != SUCCESS {
            let err = unsafe {
                let ptr = get_last_error(ret);
                CStr::from_ptr(ptr).to_str().unwrap_unchecked()
            }
            .to_string();
            return Err(err);
        }
        let modules = unsafe { slice::from_raw_parts(modules_ptr, size) };
        let modules = modules
            .iter()
            .map(|&FFIModule { start, end, name }| {
                let pathname = unsafe { CStr::from_ptr(name) }
                    .to_str()
                    .unwrap()
                    .to_string();
                Module { start, end, pathname }
            })
            .collect::<Vec<_>>();
        Ok(modules)
    }

    pub fn create_pointer_map(&self, modules: Vec<Module>) -> Result<(), String> {
        let modules = modules
            .into_iter()
            .map(|Module { start, end, pathname }| {
                let name = CString::new(pathname).unwrap();
                (start, end, name)
            })
            .collect::<Vec<_>>();
        let modules = modules
            .iter()
            .map(|&(start, end, ref name)| FFIModule { start, end, name: name.as_ptr() })
            .collect::<Vec<_>>();

        let modules_ptr = modules.as_ptr();
        let size = modules.len();
        let ret = unsafe { ptrscan_create_pointer_map(self.ptr, modules_ptr, size) };
        if ret != SUCCESS {
            let err = unsafe {
                let ptr = get_last_error(ret);
                CStr::from_ptr(ptr).to_str().unwrap_unchecked()
            };
            return Err(err.to_string());
        }

        Ok(())
    }

    pub fn create_pointer_map_file(
        &self,
        modules: Vec<Module>,
        pathname: impl AsRef<Path>,
    ) -> Result<(), String> {
        let pathname =
            CString::new(pathname.as_ref().to_str().unwrap()).map_err(|e| e.to_string())?;
        let modules = modules
            .into_iter()
            .map(|Module { start, end, pathname }| {
                let name = CString::new(pathname).unwrap();
                (start, end, name)
            })
            .collect::<Vec<_>>();
        let ffi_modules = modules
            .iter()
            .map(|&(start, end, ref name)| FFIModule { start, end, name: name.as_ptr() })
            .collect::<Vec<_>>();

        let modules_ptr = ffi_modules.as_ptr();
        let size = ffi_modules.len();
        let ret = unsafe {
            ptrscan_create_pointer_map_file(self.ptr, modules_ptr, size, pathname.as_ptr())
        };
        if ret != SUCCESS {
            let err = unsafe {
                let ptr = get_last_error(ret);
                CStr::from_ptr(ptr).to_str().unwrap_unchecked()
            };
            return Err(err.to_string());
        }

        Ok(())
    }

    pub fn load_pointer_map_file(&mut self, pathname: impl AsRef<Path>) -> Result<(), String> {
        let pathname =
            CString::new(pathname.as_ref().to_str().unwrap()).map_err(|e| e.to_string())?;
        let ret = unsafe { ptrscan_load_pointer_map_file(self.ptr, pathname.as_ptr()) };
        if ret != SUCCESS {
            let err = {
                unsafe {
                    let ptr = get_last_error(ret);
                    CStr::from_ptr(ptr).to_str().unwrap_unchecked().to_string()
                }
            };
            return Err(err);
        }

        Ok(())
    }

    pub fn scan_pointer_chain(
        &self,
        param: Param,
        pathname: impl AsRef<Path>,
    ) -> Result<(), String> {
        let Param { addr, depth, srange, lrange, node, last, max, cycle } = param;
        let param = FFIParam {
            addr,
            depth,
            srange: FFIRange { left: srange.0, right: srange.1 },
            lrange: match lrange {
                Some((a, b)) => &FFIRange { left: a, right: b },
                None => ptr::null(),
            },
            node: match node {
                Some(a) => &a,
                None => ptr::null(),
            },
            last: match last {
                Some(a) => &a,
                None => ptr::null(),
            },
            max: match max {
                Some(a) => &a,
                None => ptr::null(),
            },
            cycle,
            raw1: false,
            raw2: false,
            raw3: false,
        };

        let pathname =
            CString::new(pathname.as_ref().to_str().unwrap()).map_err(|e| e.to_string())?;
        let ret = unsafe { ptrscan_scan_pointer_chain(self.ptr, param, pathname.as_ptr()) };
        if ret != SUCCESS {
            let err = {
                unsafe {
                    let ptr = get_last_error(ret);
                    CStr::from_ptr(ptr).to_str().unwrap_unchecked().to_string()
                }
            };
            return Err(err);
        }

        Ok(())
    }

    pub fn read_memory_exact(&self, addr: usize, buf: &mut [u8]) -> Result<(), String> {
        let ret = unsafe { ptrscan_read_memory_exact(self.ptr, addr, buf.as_mut_ptr(), buf.len()) };
        if ret != SUCCESS {
            let err = {
                unsafe {
                    let ptr = get_last_error(ret);
                    CStr::from_ptr(ptr).to_str().unwrap_unchecked().to_string()
                }
            };
            return Err(err);
        }
        Ok(())
    }
}
