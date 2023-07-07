use std::{
    ffi::OsString,
    mem,
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use mach2::{
    kern_return::{kern_return_t, KERN_SUCCESS},
    libproc,
    mach_types::vm_task_entry_t,
    message::mach_msg_type_number_t,
    port::{mach_port_name_t, mach_port_t, MACH_PORT_NULL},
    traps::{mach_task_self, task_for_pid},
    vm::{mach_vm_read_overwrite, mach_vm_region, mach_vm_write},
    vm_prot::{VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_region::{vm_region_extended_info, vm_region_extended_info_data_t, vm_region_info_t, VM_REGION_EXTENDED_INFO},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::{
    vmmap64::{ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery, VirtualQueryExt},
    Error, Pid,
};

const PROC_PIDPATHINFO_MAXSIZE: usize = (libproc::PROC_PIDPATHINFO_MAXSIZE - 1) as _;

pub struct Process {
    pid: Pid,
    pub task: mach_port_t,
    pathname: PathBuf,
}

impl VirtualMemoryRead for Process {
    type Error = Error;

    fn read_at(&self, address: u64, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut out = 0;
        let result =
            unsafe { mach_vm_read_overwrite(self.task, address, buf.len() as _, buf.as_mut_ptr() as _, &mut out) };
        if result != KERN_SUCCESS {
            return Err(Error::ReadMemory(result));
        }
        Ok(out as _)
    }
}

impl VirtualMemoryWrite for Process {
    type Error = Error;

    fn write_at(&self, address: u64, buf: &[u8]) -> Result<(), Self::Error> {
        let result = unsafe { mach_vm_write(self.task, address, buf.as_ptr() as _, buf.len() as _) };
        if result != KERN_SUCCESS {
            return Err(Error::WriteMemory(result));
        }
        Ok(())
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + '_> {
        PageIter::new(self.task).map(|m| Page {
            addr: m.addr,
            size: m.size,
            count: m.count,
            info: m.info,
            pathname: proc_regionfilename(self.pid, m.addr).ok().and_then(|p| p),
        })
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        Self::o(pid).map_err(Error::OpenProcess)
    }

    fn o(pid: Pid) -> Result<Self, kern_return_t> {
        let mut task: mach_port_name_t = MACH_PORT_NULL;
        let result = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
        if result != KERN_SUCCESS {
            return Err(result);
        }
        let pathname = proc_pidpath(pid)?;
        Ok(Self { pid, task, pathname })
    }
}

#[allow(dead_code)]
struct Page {
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    count: mach_msg_type_number_t,
    info: vm_region_extended_info,
    pathname: Option<PathBuf>,
}

impl VirtualQuery for Page {
    fn start(&self) -> u64 {
        self.addr
    }

    fn end(&self) -> u64 {
        self.addr + self.size
    }

    fn size(&self) -> u64 {
        self.size
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
}

impl VirtualQueryExt for Page {
    fn tag(&self) -> u32 {
        self.info.user_tag
    }

    fn is_reserve(&self) -> bool {
        self.start() == 0xfc0000000 || self.start() == 0x1000000000
    }

    fn path(&self) -> Option<&Path> {
        self.pathname.as_deref()
    }
}

#[inline(always)]
#[allow(clippy::comparison_chain)]
fn proc_regionfilename(pid: Pid, address: u64) -> Result<Option<PathBuf>, kern_return_t> {
    unsafe {
        let mut buf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE);
        let result = libproc::proc_regionfilename(pid, address, buf.as_mut_ptr() as _, buf.capacity() as _);
        if result < 0 {
            Err(result)
        } else if result == 0 {
            Ok(None)
        } else {
            buf.set_len(result as _);
            Ok(Some(PathBuf::from(OsString::from_vec(buf))))
        }
    }
}

#[inline(always)]
fn proc_pidpath(pid: Pid) -> Result<PathBuf, kern_return_t> {
    unsafe {
        let mut buf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE);
        let result = libproc::proc_pidpath(pid, buf.as_mut_ptr() as _, buf.capacity() as _);
        if result <= 0 {
            Err(result)
        } else {
            buf.set_len(result as usize);
            Ok(PathBuf::from(OsString::from_vec(buf)))
        }
    }
}

#[allow(unused)]
struct PageRange {
    pub addr: mach_vm_address_t,
    pub size: mach_vm_size_t,
    pub count: mach_msg_type_number_t,
    pub info: vm_region_extended_info,
}

struct PageIter {
    task: vm_task_entry_t,
    addr: mach_vm_address_t,
}

impl PageIter {
    const fn new(task: mach_port_name_t) -> Self {
        Self { task, addr: 1 }
    }
}

impl Default for PageIter {
    fn default() -> Self {
        Self { task: unsafe { mach_task_self() }, addr: 1 }
    }
}

impl Iterator for PageIter {
    type Item = PageRange;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = mem::size_of::<vm_region_extended_info_data_t>() as mach_msg_type_number_t;
        let mut object_name = 0;
        let mut size = unsafe { mem::zeroed::<mach_vm_size_t>() };
        let mut info = unsafe { mem::zeroed::<vm_region_extended_info_data_t>() };

        let result = unsafe {
            mach_vm_region(
                self.task,
                &mut self.addr,
                &mut size,
                VM_REGION_EXTENDED_INFO,
                &mut info as *mut _ as vm_region_info_t,
                &mut count,
                &mut object_name,
            )
        };

        if result != KERN_SUCCESS {
            return None;
        }
        let region = PageRange { addr: self.addr, size, count, info };
        self.addr += region.size;
        Some(region)
    }
}
