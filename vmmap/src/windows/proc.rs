use core::{mem, ptr};
use std::{
    ffi::OsString,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use windows_sys::Win32::{
    Foundation::{GetLastError, FALSE, HANDLE, MAX_PATH, WIN32_ERROR},
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        Environment::GetCurrentDirectoryW,
        Memory::{
            VirtualQueryEx, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
            PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
        },
        ProcessStatus::GetMappedFileNameW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE},
    },
};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery, VirtualQueryExt};

#[derive(Debug, Clone)]
pub struct Process {
    pid: Pid,
    handle: HANDLE,
    pathname: PathBuf,
}

impl VirtualMemoryRead for Process {
    type Error = Error;

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let code =
            unsafe { ReadProcessMemory(self.handle, offset as _, buf.as_mut_ptr() as _, buf.len(), ptr::null_mut()) };
        if code == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::ReadMemory(error));
        }

        Ok(buf.len())
    }
}

impl VirtualMemoryWrite for Process {
    type Error = Error;

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<(), Self::Error> {
        let code =
            unsafe { WriteProcessMemory(self.handle, offset as _, buf.as_ptr() as _, buf.len(), ptr::null_mut()) };

        if code == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::WriteMemory(error));
        }

        Ok(())
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
                FALSE,
                pid,
            )
        };

        if handle == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::OpenProcess(error));
        }

        let mut buffer: [u16; MAX_PATH as _] = [0; MAX_PATH as _];

        if unsafe { GetCurrentDirectoryW(MAX_PATH, buffer.as_mut_ptr()) } == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::OpenProcess(error));
        }

        let pathname = PathBuf::from(wstr_to_string(&buffer));

        Ok(Self { pid, handle, pathname })
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> Box<dyn Iterator<Item = Map> + '_> {
        Box::new(MapIter::new(self.handle))
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    start: usize,
    size: usize,
    flags: u32,
    pathname: Option<PathBuf>,
}

impl VirtualQuery for Map {
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

    fn path(&self) -> Option<&Path> {
        self.pathname.as_deref()
    }
}

pub struct MapIter {
    handle: HANDLE,
    base: usize,
    tmp: [u16; MAX_PATH as usize],
}

impl MapIter {
    pub const fn new(handle: HANDLE) -> Self {
        Self { handle, base: 0, tmp: [0u16; MAX_PATH as usize] }
    }
}

impl Iterator for MapIter {
    type Item = Map;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut basic = mem::MaybeUninit::uninit();

            if VirtualQueryEx(self.handle, self.base as _, basic.as_mut_ptr(), mem::size_of::<Map>())
                != mem::size_of::<MEMORY_BASIC_INFORMATION>()
            {
                return None;
            }

            let pathname = get_path_name(self.handle, self.base, &mut self.tmp).ok();

            let info = basic.assume_init();
            self.base = info.BaseAddress as usize + info.RegionSize;

            Some(Map {
                start: info.BaseAddress as _,
                size: info.RegionSize,
                flags: info.Protect,
                pathname,
            })
        }
    }
}

#[inline(always)]
unsafe fn get_path_name(handle: HANDLE, base: usize, buf: &mut [u16; 260]) -> Result<PathBuf, WIN32_ERROR> {
    let result = GetMappedFileNameW(handle, base as _, buf.as_mut_ptr(), buf.len() as _);
    if result <= 0 {
        return Err(GetLastError());
    }
    Ok(PathBuf::from(wstr_to_string(&buf[..result as _])))
}

#[inline(always)]
fn wstr_to_string(full: &[u16]) -> OsString {
    let len = full.iter().position(|&x| x == 0).unwrap_or(full.len());
    OsString::from_wide(&full[..len])
}

impl VirtualQueryExt for Map {}
