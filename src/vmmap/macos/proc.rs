use std::fmt::Display;

use crate::vmmap::VirtualMemoryInfo;

use super::ffi;

use super::ffi_impl::{
    mach_vm_read_overwrite, mach_vm_write, proc_pidpath, proc_regionfilename, task_for_pid, MapIter,
};

use super::{Error, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

#[derive(Clone)]
pub struct Process {
    pub pid: i32,
    pub task: ffi::mach_port_t,
    pathname: String,
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, address: usize, buf: &mut [u8]) -> Result<usize> {
        let mut out: u64 = 0;
        mach_vm_read_overwrite(self.task, address as _, buf.len() as _, buf.as_mut_ptr() as _, &mut out)
            .map_err(Error::ReadMemory)?;
        Ok(out as _)
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, address: usize, buf: &[u8]) -> Result<()> {
        mach_vm_write(self.task, address as _, buf).map_err(Error::WriteMemory)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Map {
    addr: ffi::mach_vm_address_t,
    size: ffi::mach_vm_size_t,
    count: ffi::mach_msg_type_number_t,
    info: ffi::vm_region_extended_info,
    pathname: String,
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={:x}-{:x} {}",
            self.name(),
            self.start(),
            self.end(),
            Perm((self.is_read(), self.is_write(), self.is_exec()))
        )
    }
}

pub struct Perm((bool, bool, bool));

impl Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.0 .0 { "r" } else { "-" },
            if self.0 .1 { "w" } else { "-" },
            if self.0 .2 { "x" } else { "-" }
        )
    }
}

impl Map {
    fn tag(&self) -> u32 {
        self.info.user_tag
    }
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
        self.info.protection & ffi::VM_PROT_READ != 0
    }

    fn is_write(&self) -> bool {
        self.info.protection & ffi::VM_PROT_WRITE != 0
    }

    fn is_exec(&self) -> bool {
        self.info.protection & ffi::VM_PROT_EXECUTE != 0
    }

    fn is_stack(&self) -> bool {
        self.tag() == 30
    }

    fn is_heap(&self) -> bool {
        matches!(self.tag(), 1..=4 | 7..=11)
    }

    fn path(&self) -> &str {
        &self.pathname
    }

    fn name(&self) -> &str {
        let name = &self.path().rsplit_once('/').map(|s| s.1).unwrap_or_default();
        if name.len() > 16 {
            &name[..16]
        } else {
            name
        }
    }
}

impl Process {
    pub fn open(pid: i32) -> Result<Self> {
        Self::o(pid).map_err(Error::OpenProces)
    }

    fn o(pid: i32) -> Result<Self, ffi::kern_return_t> {
        let task = task_for_pid(pid)?;
        let pathname = proc_pidpath(pid)?;
        Ok(Self { pid, task, pathname })
    }

    pub fn pathname(&self) -> &str {
        &self.pathname
    }
}

impl VirtualMemoryInfo for Process {
    fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + Clone + Display + '_> {
        MapIter::new(self.task).map(move |m| Map {
            addr: m.addr,
            size: m.size,
            count: m.count,
            info: m.info,
            pathname: proc_regionfilename(self.pid, m.addr).unwrap_or_default(),
        })
    }
}
