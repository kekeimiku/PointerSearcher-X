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
            VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_FREE, MEM_IMAGE, MEM_MAPPED, PAGE_EXECUTE, PAGE_EXECUTE_READ,
            PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
        },
        ProcessStatus::GetMappedFileNameW,
        SystemInformation::GetSystemInfo,
        Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_NATIVE, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

use super::{
    Error, Pid, ProcessInfo, ProcessInfoExt, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery,
    VirtualQueryExt,
};

pub struct Process {
    pub pid: Pid,
    pub handle: HandleWrapper,
    pub pathname: PathBuf,
}

pub struct HandleWrapper(HANDLE);

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        unsafe {
            let ret = ReadProcessMemory(self.handle.0, offset as _, buf.as_mut_ptr() as _, buf.len(), ptr::null_mut());
            if ret == FALSE {
                let err = GetLastError();
                return Err(Error::ReadMemory(err));
            }
            Ok(buf.len())
        }
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: usize) -> Result<()> {
        unsafe {
            let ret = ReadProcessMemory(self.handle.0, offset as _, buf.as_mut_ptr() as _, buf.len(), ptr::null_mut());
            if ret == FALSE {
                let err = GetLastError();
                return Err(Error::ReadMemory(err));
            }
            Ok(())
        }
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<usize> {
        unsafe {
            let ret = WriteProcessMemory(self.handle.0, offset as _, buf.as_ptr() as _, buf.len(), ptr::null_mut());
            if ret == FALSE {
                let err = GetLastError();
                return Err(Error::WriteMemory(err));
            }
            Ok(buf.len())
        }
    }

    fn write_all_at(&self, buf: &[u8], offset: usize) -> Result<()> {
        unsafe {
            let ret = WriteProcessMemory(self.handle.0, offset as _, buf.as_ptr() as _, buf.len(), ptr::null_mut());
            if ret == FALSE {
                let err = GetLastError();
                return Err(Error::WriteMemory(err));
            }
            Ok(())
        }
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self> {
        unsafe { Self::_open(pid) }.map_err(Error::OpenProcess)
    }

    unsafe fn _open(pid: Pid) -> Result<Self, WIN32_ERROR> {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            FALSE,
            pid,
        );

        if handle == 0 {
            let err = GetLastError();
            return Err(err);
        }

        let mut buf = Vec::with_capacity(MAX_PATH as usize);
        let mut size = MAX_PATH;
        let ret = QueryFullProcessImageNameW(handle, PROCESS_NAME_NATIVE, buf.as_mut_ptr(), &mut size);
        if ret == 0 {
            let err = GetLastError();
            return Err(err);
        }
        buf.set_len(size as usize);
        let pathname = PathBuf::from(OsString::from_wide(&buf));
        Ok(Self { pid, handle: HandleWrapper(handle), pathname })
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> impl Iterator<Item = Result<Mapping>> {
        let sys_info = unsafe {
            let mut sys_info = mem::MaybeUninit::uninit();
            GetSystemInfo(sys_info.as_mut_ptr());
            sys_info.assume_init()
        };
        let min_addr = sys_info.lpMinimumApplicationAddress as usize;
        let max_addr = sys_info.lpMaximumApplicationAddress as usize;
        Iter::new(self.handle.0, min_addr, max_addr).map(|x| x.map_err(Error::QueryMapping))
    }
}

impl ProcessInfoExt for Process {
    fn handle(&self) -> isize {
        self.handle.0
    }
}

pub struct Mapping {
    pub addr: usize,
    pub size: usize,
    pub protect: u32,
    pub state: u32,
    pub r#type: u32,
    pub name: Option<String>,
}

impl VirtualQuery for Mapping {
    fn start(&self) -> usize {
        self.addr
    }

    fn end(&self) -> usize {
        self.addr + self.size
    }

    fn size(&self) -> usize {
        self.size
    }

    fn is_read(&self) -> bool {
        self.protect
            & (PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY
                | PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY)
            != 0
    }

    fn is_write(&self) -> bool {
        self.protect & (PAGE_EXECUTE_READWRITE | PAGE_READWRITE) != 0
    }

    fn is_exec(&self) -> bool {
        self.protect & (PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl VirtualQueryExt for Mapping {
    fn is_free(&self) -> bool {
        self.state == MEM_FREE
    }

    fn is_guard(&self) -> bool {
        self.protect & PAGE_GUARD != 0
    }

    fn m_type(&self) -> u32 {
        self.r#type
    }

    fn m_state(&self) -> u32 {
        self.state
    }

    fn m_protect(&self) -> u32 {
        self.protect
    }
}

struct Iter {
    handle: HANDLE,
    min_addr: usize,
    max_addr: usize,
}

impl Iter {
    const fn new(handle: HANDLE, min_addr: usize, max_addr: usize) -> Self {
        Self { handle, min_addr, max_addr }
    }
}

impl Iterator for Iter {
    type Item = Result<Mapping, WIN32_ERROR>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.min_addr >= self.max_addr {
                return None;
            }
            let mut info_uninit = mem::MaybeUninit::uninit();
            let ret = VirtualQueryEx(
                self.handle,
                self.min_addr as _,
                info_uninit.as_mut_ptr(),
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );
            if ret == 0 {
                let err = GetLastError();
                return Some(Err(err));
            }
            let info = info_uninit.assume_init();
            let name = match info.Type {
                MEM_IMAGE | MEM_MAPPED => get_mapped_file_name_w(self.handle, self.min_addr).ok(),
                _ => None,
            };

            let addr = self.min_addr;
            self.min_addr += info.RegionSize;
            Some(Ok(Mapping {
                addr,
                size: info.RegionSize,
                protect: info.Protect,
                state: info.State,
                r#type: info.Type,
                name,
            }))
        }
    }
}

unsafe fn get_mapped_file_name_w(handle: HANDLE, base: usize) -> Result<String, WIN32_ERROR> {
    let mut buf = Vec::with_capacity(MAX_PATH as usize);
    let ret = GetMappedFileNameW(handle, base as _, buf.as_mut_ptr(), MAX_PATH);
    if ret == 0 {
        let err = GetLastError();
        return Err(err);
    }
    buf.set_len(ret as usize);
    Ok(OsString::from_wide(&buf).into_string().unwrap())
}
