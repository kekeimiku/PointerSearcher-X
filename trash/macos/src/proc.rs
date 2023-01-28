use crate::{
    map::{Dyld, Map, MapIter},
    os,
};

use super::ffi;

type Result<T, E = ffi::kern_return_t> = core::result::Result<T, E>;

pub trait Read {
    fn read_at(&self, addr: usize, size: usize) -> Result<Vec<u8>>;
}

pub trait Write {
    fn write_at(&self, addr: usize, buf: &[u8]) -> Result<()>;
}

pub struct Process {
    pub pid: i32,
    pub task: ffi::mach_port_t,
}

impl AsRef<Process> for Process {
    fn as_ref(&self) -> &Process {
        self
    }
}

pub trait ProcessExt<M = Self, D = Self>: Read + Write {
    fn get_map_iter(&self) -> impl Iterator<Item = M>;
    fn get_dyld_iter(&self) -> impl Iterator<Item = D>;
    fn write_memory(&self, addr: usize, buf: &[u8]) -> Result<()>;
    fn read_memory(&self, addr: usize, size: usize) -> Result<Vec<u8>>;
    fn ge_dyld_collect(&self) -> Vec<M>;
    fn get_map_collect(&self) -> Vec<D>;
}

impl<T> ProcessExt<Map, Dyld> for T
where
    T: Read + Write + AsRef<Process>,
{
    default fn get_map_iter(&self) -> impl Iterator<Item = Map> {
        MapIter::new(self.as_ref().task)
    }

    default fn get_dyld_iter(&self) -> impl Iterator<Item = Dyld> {
        Dyld.into_iter()
    }

    default fn write_memory(&self, addr: usize, buf: &[u8]) -> Result<()> {
        self.as_ref().write_at(addr, buf)
    }

    default fn read_memory(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        self.as_ref().read_at(addr, size)
    }

    default fn ge_dyld_collect(&self) -> Vec<Map> {
        self.get_map_iter().collect()
    }

    default fn get_map_collect(&self) -> Vec<Dyld> {
        self.get_dyld_iter().collect()
    }
}

impl Process {
    pub fn open(pid: i32) -> Result<Self> {
        let task = os::task_for_pid(pid)?;
        Ok(Self { pid, task })
    }
}

impl Read for Process {
    fn read_at(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size];
        os::mach_vm_read_overwrite(self.task, addr as _, &mut buf)?;
        Ok(buf)
    }
}

impl Write for Process {
    fn write_at(&self, addr: usize, buf: &[u8]) -> Result<()> {
        os::mach_vm_write(self.task, addr as _, buf)
    }
}
