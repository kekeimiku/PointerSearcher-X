use core::ops::Range;
use std::path::Path;

use machx::{
    kern_return::KERN_SUCCESS,
    port::{mach_port_name_t, MACH_PORT_NULL},
    traps::{mach_task_self, task_for_pid},
    vm::mach_vm_read_overwrite,
};

use super::{
    dump::{create_pointer_map, create_pointer_map_file},
    info::{list_image_maps, list_unknown_maps},
    Error, ModuleMap, PointerMap,
};

pub struct Process {
    pid: i32,
    task: mach_port_name_t,
}

impl Process {
    pub fn attach(pid: i32) -> Result<Self, Error> {
        let mut task: mach_port_name_t = MACH_PORT_NULL;
        let kr = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
        if kr != KERN_SUCCESS {
            return Err(Error::AttachProcess(kr));
        }
        Ok(Self { pid, task })
    }

    pub fn list_image_maps(&self) -> Result<ModuleMap<usize, String>, Error> {
        unsafe { list_image_maps(self.pid, self.task) }.map_err(Error::QueryProcess)
    }

    pub fn list_unknown_maps(&self) -> Result<Vec<Range<usize>>, Error> {
        unsafe { list_unknown_maps(self.pid) }.map_err(Error::QueryProcess)
    }

    pub fn create_pointer_map_file(
        &self,
        module_maps: &ModuleMap<usize, String>,
        unknown_maps: &[Range<usize>],
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        create_pointer_map_file(self.task, module_maps, unknown_maps, path)
    }

    pub fn create_pointer_map(
        &self,
        module_maps: ModuleMap<usize, String>,
        unknown_maps: &[Range<usize>],
    ) -> Result<PointerMap, Error> {
        create_pointer_map(self.task, module_maps, unknown_maps)
    }

    pub fn read_memory_exact(&self, addr: usize, buf: &mut [u8]) -> Result<(), Error> {
        let mut outsize = 0;
        let kr = unsafe {
            mach_vm_read_overwrite(
                self.task,
                addr as u64,
                buf.len() as u64,
                buf.as_mut_ptr() as u64,
                &mut outsize,
            )
        };
        if kr != KERN_SUCCESS {
            return Err(Error::ReadMemory(kr));
        }
        Ok(())
    }
}
