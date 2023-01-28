use core::mem;

use super::{ffi, os};

pub trait MapExt {
    fn start(&self) -> u64;
    fn end(&self) -> u64;
    fn size(&self) -> u64;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
}

// TODO Return similar linux-style maps? `start-end perm pathname/dyld`
#[derive(Debug)]
#[allow(unused)]
pub struct Map {
    pub(crate) addr: ffi::mach_vm_address_t,
    pub(crate) size: ffi::mach_vm_size_t,
    pub(crate) count: ffi::mach_msg_type_number_t,
    pub(crate) info: ffi::vm_region_extended_info,
}

pub struct MapIter {
    task: ffi::vm_task_entry_t,
    addr: ffi::mach_vm_address_t,
}

impl MapIter {
    pub fn new(task: ffi::mach_port_name_t) -> Self {
        Self { task, addr: 1 }
    }
}

impl Default for MapIter {
    fn default() -> Self {
        Self { task: unsafe { ffi::mach_task_self() }, addr: 1 }
    }
}

// TODO Maybe shouldnot use iterator?
impl Iterator for MapIter {
    type Item = Map;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = mem::size_of::<ffi::vm_region_extended_info_data_t>() as ffi::mach_msg_type_number_t;
        let mut object_name: ffi::mach_port_t = 0;
        let mut size = unsafe { mem::zeroed::<ffi::mach_vm_size_t>() };
        let mut info = unsafe { mem::zeroed::<ffi::vm_region_extended_info_data_t>() };
        let result: ffi::kern_return_t = os::mach_vm_region(
            self.task,
            &mut self.addr,
            &mut size,
            ffi::VM_REGION_EXTENDED_INFO,
            &mut info as *mut ffi::vm_region_extended_info_data_t as ffi::vm_region_info_t,
            &mut count,
            &mut object_name,
        );

        if result != ffi::KERN_SUCCESS {
            return None;
        }
        let region = Map { addr: self.addr, size, count, info };
        self.addr += region.size;
        Some(region)
    }
}

impl MapExt for Map {
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
        self.info.protection & ffi::VM_PROT_READ != 0
    }

    fn is_write(&self) -> bool {
        self.info.protection & ffi::VM_PROT_WRITE != 0
    }

    fn is_exec(&self) -> bool {
        self.info.protection & ffi::VM_PROT_EXECUTE != 0
    }
}

impl Map {
    pub fn tag(&self) -> u32 {
        self.info.user_tag
    }

    pub fn dyld(&self) -> &str {
        todo!()
    }

    pub fn pathname(&self, pid: i32) -> String {
        let buf = os::proc_regionfilename(pid, self.start()).unwrap_or_default();
        String::from_utf8_lossy(&buf).to_string()
    }
}

pub struct Dyld;

impl Iterator for Dyld {
    type Item = Dyld;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
