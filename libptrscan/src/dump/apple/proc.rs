use std::{io::Error, path::Path};

use machx::{
    kern_return::KERN_SUCCESS,
    port::{mach_port_name_t, MACH_PORT_NULL},
    traps::{mach_task_self, task_for_pid},
};

use super::{
    dump::{create_pointer_map, create_pointer_map_file, read_memory_exact},
    info::{kern_error, list_image_maps, list_unknown_maps},
    PointerMap, RangeMap, RangeSet,
};

pub struct Process {
    pid: i32,
    task: mach_port_name_t,
}

impl Process {
    pub fn attach(pid: i32) -> Result<Self, Error> {
        let mut task: mach_port_name_t = MACH_PORT_NULL;
        unsafe {
            let kr = task_for_pid(mach_task_self(), pid, &mut task);
            if kr != KERN_SUCCESS {
                return Err(kern_error(kr));
            }
        }
        Ok(Self { pid, task })
    }

    pub fn list_image_maps(&self) -> Result<RangeMap<usize, String>, Error> {
        unsafe { list_image_maps(self.pid, self.task) }
    }

    pub fn list_unknown_maps(&self) -> Result<RangeSet<usize>, Error> {
        unsafe { list_unknown_maps(self.pid) }
    }

    pub fn create_pointer_map_file(
        &self,
        module_maps: RangeMap<usize, String>,
        unknown_maps: RangeSet<usize>,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        create_pointer_map_file(self.task, module_maps, unknown_maps, path)
    }

    pub fn create_pointer_map(
        &self,
        module_maps: RangeMap<usize, String>,
        unknown_maps: RangeSet<usize>,
    ) -> Result<PointerMap, Error> {
        create_pointer_map(self.task, module_maps, unknown_maps)
    }

    pub fn read_memory_exact(&self, addr: usize, buf: &mut [u8]) -> Result<(), Error> {
        unsafe { read_memory_exact(self.task, addr, buf) }
    }
}
