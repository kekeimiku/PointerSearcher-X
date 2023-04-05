use core::mem;
use std::{
    ffi::OsStr,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use mach2::{
    kern_return::{kern_return_t, KERN_SUCCESS},
    libproc::{self, PROC_PIDPATHINFO_MAXSIZE},
    mach_types::vm_task_entry_t,
    message::mach_msg_type_number_t,
    port::{mach_port_name_t, mach_port_t, MACH_PORT_NULL},
    traps::{mach_task_self, task_for_pid},
    vm::{mach_vm_read_overwrite, mach_vm_region, mach_vm_write},
    vm_prot::{VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_region::{vm_region_extended_info, vm_region_extended_info_data_t, vm_region_info_t, VM_REGION_EXTENDED_INFO},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

const MAX_PATH: usize = (PROC_PIDPATHINFO_MAXSIZE - 1) as _;

#[derive(Clone)]
pub struct Process {
    pid: Pid,
    task: mach_port_t,
    pathname: PathBuf,
}

impl VirtualMemoryRead for Process {
    type Error = Error;

    fn read_at(&self, address: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut out = 0;
        let result =
            unsafe { mach_vm_read_overwrite(self.task, address as _, buf.len() as _, buf.as_mut_ptr() as _, &mut out) };
        if result != KERN_SUCCESS {
            return Err(Error::ReadMemory(result));
        }
        Ok(out as _)
    }
}

impl VirtualMemoryWrite for Process {
    type Error = Error;

    fn write_at(&self, address: usize, buf: &[u8]) -> Result<(), Self::Error> {
        let result = unsafe { mach_vm_write(self.task, address as _, buf.as_ptr() as _, buf.len() as _) };
        if result != KERN_SUCCESS {
            return Err(Error::WriteMemory(result));
        }
        Ok(())
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> i32 {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        Self::o(pid).map_err(Error::OpenProcess)
    }

    fn o(pid: Pid) -> Result<Self, kern_return_t> {
        let mut buf = [0_u8; MAX_PATH];
        let mut task: mach_port_name_t = MACH_PORT_NULL;
        let result = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
        if result != KERN_SUCCESS {
            return Err(result);
        }
        let pathname = proc_pidpath(pid, &mut buf)?;
        Ok(Self { pid, task, pathname })
    }

    pub fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + Clone + '_> {
        let mut buf = [0_u8; MAX_PATH];
        MapIter::new(self.task).map(move |m| Map {
            addr: m.addr,
            size: m.size,
            count: m.count,
            info: m.info,
            pathname: proc_regionfilename(self.pid, m.addr, &mut buf).ok(),
        })
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Map {
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    count: mach_msg_type_number_t,
    info: vm_region_extended_info,
    pathname: Option<PathBuf>,
}

impl VirtualQuery for Map {
    fn size(&self) -> usize {
        self.size as _
    }

    fn start(&self) -> usize {
        self.addr as _
    }

    fn end(&self) -> usize {
        (self.addr + self.size) as _
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

    fn is_stack(&self) -> bool {
        self.info.user_tag == 30
    }

    fn is_heap(&self) -> bool {
        matches!(self.info.user_tag, 1..=4 | 7..=11)
    }

    fn path(&self) -> Option<&Path> {
        self.pathname.as_deref()
    }
}

#[inline(always)]
fn proc_regionfilename(pid: Pid, address: u64, buf: &mut [u8]) -> Result<PathBuf, kern_return_t> {
    let result = unsafe { libproc::proc_regionfilename(pid, address, buf.as_mut_ptr() as _, buf.len() as _) };
    if result <= 0 {
        Err(result)
    } else {
        Ok(PathBuf::from(OsStr::from_bytes(&buf[..result as _])))
    }
}

#[inline(always)]
fn proc_pidpath(pid: Pid, buf: &mut [u8]) -> Result<PathBuf, kern_return_t> {
    let result = unsafe { libproc::proc_pidpath(pid, buf.as_mut_ptr() as _, buf.len() as _) };
    if result <= 0 {
        Err(result)
    } else {
        Ok(PathBuf::from(OsStr::from_bytes(&buf[..result as _])))
    }
}

#[allow(unused)]
pub struct MapRange {
    pub addr: mach_vm_address_t,
    pub size: mach_vm_size_t,
    pub count: mach_msg_type_number_t,
    pub info: vm_region_extended_info,
}

struct MapIter {
    task: vm_task_entry_t,
    addr: mach_vm_address_t,
}

impl MapIter {
    const fn new(task: mach_port_name_t) -> Self {
        Self { task, addr: 1 }
    }
}

impl Default for MapIter {
    fn default() -> Self {
        Self { task: unsafe { mach_task_self() }, addr: 1 }
    }
}

impl Iterator for MapIter {
    type Item = MapRange;

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
        let region = MapRange { addr: self.addr, size, count, info };
        self.addr += region.size;
        Some(region)
    }
}
