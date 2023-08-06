use std::{
    ffi::OsString,
    mem,
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use machx::{
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
    Error, Pid, ProcessInfo, ProcessInfoExt, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery, VirtualQueryExt,
};

const PROC_PIDPATHINFO_MAXSIZE: usize = (libproc::PROC_PIDPATHINFO_MAXSIZE - 1) as usize;

pub struct Process {
    pid: Pid,
    task: mach_port_t,
    pathname: PathBuf,
}

impl VirtualMemoryRead for Process {
    type Error = Error;

    fn read_at(&self, buf: &mut [u8], address: usize) -> Result<usize, Self::Error> {
        unsafe {
            let mut out = 0;
            let result =
                mach_vm_read_overwrite(self.task, address as u64, buf.len() as _, buf.as_mut_ptr() as _, &mut out);
            if result != KERN_SUCCESS {
                return Err(Error::ReadMemory(result));
            }
            Ok(out as _)
        }
    }
}

impl VirtualMemoryWrite for Process {
    type Error = Error;

    fn write_at(&self, buf: &[u8], address: usize) -> Result<(), Self::Error> {
        unsafe {
            let result = mach_vm_write(self.task, address as u64, buf.as_ptr() as _, buf.len() as _);
            if result != KERN_SUCCESS {
                return Err(Error::WriteMemory(result));
            }
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub struct Page {
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    count: mach_msg_type_number_t,
    info: vm_region_extended_info,
    pathname: Option<PathBuf>,
}

impl VirtualQuery for Page {
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

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> Box<dyn Iterator<Item = Page> + '_> {
        Box::new(PageIter::new(self.task).map(|m| Page {
            addr: m.addr,
            size: m.size,
            count: m.count,
            info: m.info,
            pathname: proc_regionfilename(self.pid, m.addr).ok(),
        }))
    }
}

impl ProcessInfoExt for Process {
    fn task(&self) -> u32 {
        self.task
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        unsafe {
            || -> _ {
                let mut task: mach_port_name_t = MACH_PORT_NULL;
                let result = task_for_pid(mach_task_self(), pid, &mut task);
                if result != KERN_SUCCESS {
                    return Err(result);
                }
                let pathname = proc_pidpath(pid)?;
                Ok(Self { pid, task, pathname })
            }()
            .map_err(Error::OpenProcess)
        }
    }
}

#[inline(always)]
#[allow(clippy::comparison_chain)]
fn proc_regionfilename(pid: Pid, address: u64) -> Result<PathBuf, kern_return_t> {
    unsafe {
        let mut buf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE);
        let result = libproc::proc_regionfilename(pid, address, buf.as_mut_ptr() as _, buf.capacity() as _);
        if result <= 0 {
            Err(result)
        } else {
            buf.set_len(result as usize);
            Ok(PathBuf::from(OsString::from_vec(buf)))
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
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    count: mach_msg_type_number_t,
    info: vm_region_extended_info,
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

impl Iterator for PageIter {
    type Item = PageRange;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut count = mem::size_of::<vm_region_extended_info_data_t>() as mach_msg_type_number_t;
            let mut object_name = 0;
            let mut size = mem::zeroed::<mach_vm_size_t>();
            let mut info = mem::zeroed::<vm_region_extended_info_data_t>();

            let result = mach_vm_region(
                self.task,
                &mut self.addr,
                &mut size,
                VM_REGION_EXTENDED_INFO,
                &mut info as *mut _ as vm_region_info_t,
                &mut count,
                &mut object_name,
            );

            if result != KERN_SUCCESS {
                return None;
            }
            let region = PageRange { addr: self.addr, size, count, info };
            self.addr += region.size;
            Some(region)
        }
    }
}
