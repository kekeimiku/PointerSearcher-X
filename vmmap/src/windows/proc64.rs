use std::{
    ffi::OsString,
    mem,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
    ptr,
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, MAX_PATH, WIN32_ERROR},
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        Memory::{
            VirtualQueryEx, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
            PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
        },
        ProcessStatus::GetMappedFileNameW,
        Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_NATIVE, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

use super::{Error, Pid, ProcessInfo, ProcessInfoExt, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub struct Process {
    pub pid: Pid,
    pub handle: HandleInner,
    pub pathname: PathBuf,
}

pub struct HandleInner(HANDLE);

impl Drop for HandleInner {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        unsafe {
            let code = ReadProcessMemory(self.handle.0, offset as _, buf.as_mut_ptr() as _, buf.len(), ptr::null_mut());
            if code == 0 {
                let error = GetLastError();
                return Err(Error::ReadMemory(error));
            }
            Ok(buf.len())
        }
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: usize) -> Result<(), Error> {
        unsafe {
            let code = ReadProcessMemory(self.handle.0, offset as _, buf.as_mut_ptr() as _, buf.len(), ptr::null_mut());
            if code == 0 {
                let error = GetLastError();
                return Err(Error::ReadMemory(error));
            }
            Ok(())
        }
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<usize, Error> {
        unsafe {
            let code = WriteProcessMemory(self.handle.0, offset as _, buf.as_ptr() as _, buf.len(), ptr::null_mut());
            if code == 0 {
                let error = GetLastError();
                return Err(Error::WriteMemory(error));
            }
            Ok(buf.len())
        }
    }

    fn write_all_at(&self, buf: &[u8], offset: usize) -> Result<(), Error> {
        unsafe {
            let code = WriteProcessMemory(self.handle.0, offset as _, buf.as_ptr() as _, buf.len(), ptr::null_mut());
            if code == 0 {
                let error = GetLastError();
                return Err(Error::WriteMemory(error));
            }
            Ok(())
        }
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        unsafe {
            || -> _ {
                let handle = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
                    FALSE,
                    pid,
                );

                if handle == 0 {
                    let error = GetLastError();
                    return Err(error);
                }

                let mut buffer = [0; MAX_PATH as _];
                let mut lpdwsize = MAX_PATH;

                let result =
                    QueryFullProcessImageNameW(handle, PROCESS_NAME_NATIVE, buffer.as_mut_ptr(), &mut lpdwsize);
                if result == 0 {
                    let error = GetLastError();
                    return Err(error);
                }

                let pathname = PathBuf::from(OsString::from_wide(&buffer[..lpdwsize as _]));
                Ok(Self { pid, handle: HandleInner(handle), pathname })
            }()
            .map_err(Error::OpenProcess)
        }
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> impl Iterator<Item = Page> {
        #[inline]
        fn skip_last<T>(mut iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
            let last = iter.next();
            iter.scan(last, |state, item| mem::replace(state, Some(item)))
        }
        skip_last(Iter::new(self.handle.0).skip(1))
    }
}

impl ProcessInfoExt for Process {
    fn handle(&self) -> isize {
        self.handle.0
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub start: usize,
    pub size: usize,
    pub flags: u32,
    pub pathname: Option<String>,
}

impl VirtualQuery for Page {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.start + self.size
    }

    fn size(&self) -> usize {
        self.size
    }

    fn is_read(&self) -> bool {
        self.flags
            & (PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY
                | PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY)
            != 0
    }

    fn is_write(&self) -> bool {
        self.flags & (PAGE_EXECUTE_READWRITE | PAGE_READWRITE) != 0
    }

    fn is_exec(&self) -> bool {
        self.flags & (PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0
    }

    fn name(&self) -> Option<&str> {
        self.pathname.as_deref()
    }
}

struct Iter {
    handle: HANDLE,
    base: usize,
}

impl Iter {
    const fn new(handle: HANDLE) -> Self {
        Self { handle, base: 0 }
    }
}

impl Iterator for Iter {
    type Item = Page;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut basic = mem::MaybeUninit::uninit();

            if VirtualQueryEx(
                self.handle,
                self.base as _,
                basic.as_mut_ptr(),
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            ) == 0
            {
                return None;
            }

            let pathname = get_mapped_file_name_w(self.handle, self.base).ok();

            let info = basic.assume_init();
            self.base = info.BaseAddress as usize + info.RegionSize;

            Some(Page {
                start: info.BaseAddress as _,
                size: info.RegionSize as _,
                flags: info.Protect,
                pathname,
            })
        }
    }
}

#[inline(always)]
unsafe fn get_mapped_file_name_w(handle: HANDLE, base: usize) -> Result<String, WIN32_ERROR> {
    let mut buf = [0; MAX_PATH as _];
    let result = GetMappedFileNameW(handle, base as _, buf.as_mut_ptr(), buf.len() as _);
    if result == 0 {
        return Err(GetLastError());
    }
    Ok(OsString::from_wide(&buf[..result as _]).into_string().unwrap())
}
