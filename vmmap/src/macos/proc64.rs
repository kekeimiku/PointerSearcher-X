use std::{
    ffi::OsString,
    mem,
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use machx::{
    kern_return::{kern_return_t, KERN_INVALID_ADDRESS, KERN_SUCCESS},
    libproc::{proc_pidpath, proc_regionfilename, PROC_PIDPATHINFO_MAXSIZE},
    mach_types::vm_task_entry_t,
    port::{mach_port_name_t, mach_port_t, MACH_PORT_NULL},
    traps::{mach_task_self, task_for_pid},
    vm::{mach_vm_read_overwrite, mach_vm_region, mach_vm_write},
    vm_prot::{VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_region::{
        vm_region_extended_info, vm_region_info_t, SM_COW, SM_EMPTY, SM_PRIVATE, SM_PRIVATE_ALIASED, SM_SHARED,
        SM_SHARED_ALIASED, SM_TRUESHARED, VM_REGION_EXTENDED_INFO,
    },
    vm_statistics::{
        VM_MEMORY_DYLD, VM_MEMORY_DYLD_MALLOC, VM_MEMORY_DYLIB, VM_MEMORY_GUARD, VM_MEMORY_MALLOC,
        VM_MEMORY_MALLOC_HUGE, VM_MEMORY_MALLOC_LARGE, VM_MEMORY_MALLOC_LARGE_REUSABLE, VM_MEMORY_MALLOC_LARGE_REUSED,
        VM_MEMORY_MALLOC_NANO, VM_MEMORY_MALLOC_SMALL, VM_MEMORY_MALLOC_TINY, VM_MEMORY_REALLOC, VM_MEMORY_SBRK,
        VM_MEMORY_STACK,
    },
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::{
    Error, Pid, ProcessInfo, ProcessInfoExt, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery,
    VirtualQueryExt,
};

pub struct Process {
    pid: Pid,
    task: mach_port_t,
    pathname: PathBuf,
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        unsafe {
            let mut outsize = 0;
            let ret =
                mach_vm_read_overwrite(self.task, offset as u64, buf.len() as _, buf.as_mut_ptr() as _, &mut outsize);
            if ret != KERN_SUCCESS {
                return Err(Error::ReadMemory(ret));
            }
            Ok(outsize as usize)
        }
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: usize) -> Result<()> {
        unsafe {
            let mut outsize = 0;
            let ret =
                mach_vm_read_overwrite(self.task, offset as u64, buf.len() as _, buf.as_mut_ptr() as _, &mut outsize);
            if ret != KERN_SUCCESS {
                return Err(Error::ReadMemory(ret));
            }
            Ok(())
        }
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<usize> {
        unsafe {
            let ret = mach_vm_write(self.task, offset as u64, buf.as_ptr() as _, buf.len() as _);
            if ret != KERN_SUCCESS {
                return Err(Error::WriteMemory(ret));
            }
            Ok(buf.len())
        }
    }

    fn write_all_at(&self, buf: &[u8], offset: usize) -> Result<()> {
        unsafe {
            let ret = mach_vm_write(self.task, offset as u64, buf.as_ptr() as _, buf.len() as _);
            if ret != KERN_SUCCESS {
                return Err(Error::WriteMemory(ret));
            }
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub struct Mapping {
    pub addr: mach_vm_address_t,
    pub size: mach_vm_size_t,
    pub info: vm_region_extended_info,
    pub pathname: Option<String>,
}

impl VirtualQuery for Mapping {
    fn start(&self) -> usize {
        self.addr as usize
    }

    fn end(&self) -> usize {
        (self.addr + self.size) as usize
    }

    fn size(&self) -> usize {
        self.size as usize
    }

    fn is_read(&self) -> bool {
        self.info.protection & VM_PROT_READ != 0
    }

    fn is_write(&self) -> bool {
        self.info.protection & VM_PROT_WRITE != 0
    }

    fn is_exec(&self) -> bool {
        self.info.protection & VM_PROT_EXECUTE != 0
    }

    fn name(&self) -> Option<&str> {
        self.pathname.as_deref()
    }
}

impl VirtualQueryExt for Mapping {
    fn share_mode(&self) -> u8 {
        self.info.share_mode
    }

    fn user_tag(&self) -> u32 {
        self.info.user_tag
    }
}

// TODO enum all mach/vm_statistics.h
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UserTag {
    Malloc,
    MallocSmall,
    MallocLarge,
    MallocHuge,
    Sbrk,
    Realloc,
    MallocTiny,
    MallocLargeReusable,
    MallocLargeReused,
    Stack,
    Guard,
    MallocNano,
    Dylib,
    Dyld,
    DyldMalloc,
    Unknown(u32),
}

impl core::fmt::Display for UserTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl From<u32> for UserTag {
    fn from(value: u32) -> Self {
        match value {
            VM_MEMORY_MALLOC => UserTag::Malloc,
            VM_MEMORY_MALLOC_SMALL => UserTag::MallocSmall,
            VM_MEMORY_MALLOC_LARGE => UserTag::MallocLarge,
            VM_MEMORY_MALLOC_HUGE => UserTag::MallocHuge,
            VM_MEMORY_SBRK => UserTag::Sbrk,
            VM_MEMORY_REALLOC => UserTag::Realloc,
            VM_MEMORY_MALLOC_TINY => UserTag::MallocTiny,
            VM_MEMORY_MALLOC_LARGE_REUSABLE => UserTag::MallocLargeReusable,
            VM_MEMORY_MALLOC_LARGE_REUSED => UserTag::MallocLargeReused,
            VM_MEMORY_MALLOC_NANO => UserTag::MallocNano,
            VM_MEMORY_STACK => UserTag::Stack,
            VM_MEMORY_GUARD => UserTag::Guard,
            VM_MEMORY_DYLIB => UserTag::Dylib,
            VM_MEMORY_DYLD => UserTag::Dyld,
            VM_MEMORY_DYLD_MALLOC => UserTag::DyldMalloc,
            _ => UserTag::Unknown(value),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShareMode {
    COW,
    PRV,
    NUL,
    ALI,
    SHM,
    P_A,
    S_A,
    LPG,
    UNK(u8),
}

impl core::fmt::Display for ShareMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl From<u8> for ShareMode {
    fn from(value: u8) -> Self {
        match value {
            SM_COW => ShareMode::COW,
            SM_PRIVATE => ShareMode::PRV,
            SM_EMPTY => ShareMode::NUL,
            SM_SHARED => ShareMode::ALI,
            SM_TRUESHARED => ShareMode::SHM,
            SM_PRIVATE_ALIASED => ShareMode::P_A,
            SM_SHARED_ALIASED => ShareMode::S_A,
            _ => ShareMode::UNK(value),
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

    fn get_maps(&self) -> impl Iterator<Item = Result<Mapping>> {
        Iter::new(self.task).map(|m| from_page(self.pid, m).map_err(Error::QueryMapping))
    }
}

#[inline]
fn from_page(pid: Pid, pg: Result<MappingBase, kern_return_t>) -> Result<Mapping, kern_return_t> {
    let pg = pg?;
    Ok(Mapping {
        addr: pg.addr,
        size: pg.size,
        info: pg.info,
        pathname: unsafe { regionfilename(pid, pg.addr) }?,
    })
}

impl ProcessInfoExt for Process {
    fn task(&self) -> u32 {
        self.task
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self> {
        unsafe { Self::_open(pid) }.map_err(Error::OpenProcess)
    }

    unsafe fn _open(pid: Pid) -> Result<Self, kern_return_t> {
        let mut task: mach_port_name_t = MACH_PORT_NULL;
        let ret = task_for_pid(mach_task_self(), pid, &mut task);
        if ret != KERN_SUCCESS {
            return Err(ret);
        }
        let pathname = pidpath(pid)?;
        Ok(Self { pid, task, pathname })
    }
}

unsafe fn regionfilename(pid: Pid, address: u64) -> Result<Option<String>, kern_return_t> {
    let mut buf = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE as usize - 1);
    let ret = proc_regionfilename(pid, address, buf.as_mut_ptr() as _, buf.capacity() as _);
    #[allow(clippy::comparison_chain)]
    if ret < 0 {
        Err(ret)
    } else if ret == 0 {
        Ok(None)
    } else {
        buf.set_len(ret as usize);
        Ok(Some(String::from_utf8_unchecked(buf)))
    }
}

unsafe fn pidpath(pid: Pid) -> Result<PathBuf, kern_return_t> {
    let mut buf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE as usize - 1);
    let ret = proc_pidpath(pid, buf.as_mut_ptr() as _, buf.capacity() as _);
    if ret <= 0 {
        Err(ret)
    } else {
        buf.set_len(ret as usize);
        Ok(PathBuf::from(OsString::from_vec(buf)))
    }
}

struct MappingBase {
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    info: vm_region_extended_info,
}

struct Iter {
    task: vm_task_entry_t,
    addr: mach_vm_address_t,
}

impl Iter {
    const fn new(task: mach_port_name_t) -> Self {
        Self { task, addr: 1 }
    }
}

impl Iterator for Iter {
    type Item = Result<MappingBase, kern_return_t>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut object_name = 0;
            let mut size = mem::zeroed::<mach_vm_size_t>();
            let mut info_uninit = mem::MaybeUninit::uninit();
            let ret = mach_vm_region(
                self.task,
                &mut self.addr,
                &mut size,
                VM_REGION_EXTENDED_INFO,
                &mut info_uninit as *mut _ as vm_region_info_t,
                &mut vm_region_extended_info::count(),
                &mut object_name,
            );

            if ret == KERN_INVALID_ADDRESS {
                return None;
            }

            if ret != KERN_SUCCESS {
                return Some(Err(ret));
            }
            let info = info_uninit.assume_init();
            let region = MappingBase { addr: self.addr, size, info };
            self.addr += region.size;
            Some(Ok(region))
        }
    }
}
