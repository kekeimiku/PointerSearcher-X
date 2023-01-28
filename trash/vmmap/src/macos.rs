#[allow(non_camel_case_types)]
mod ffi {
    pub type natural_t = ::core::ffi::c_uint;
    pub type integer_t = ::core::ffi::c_int;
    pub type mach_port_t = ::core::ffi::c_uint;
    pub type mach_port_name_t = natural_t;
    pub type vm_task_entry_t = mach_port_t;
    pub type mach_vm_address_t = u64;
    pub type mach_vm_size_t = u64;
    pub type vm_region_flavor_t = ::core::ffi::c_int;
    pub type vm_region_info_t = *mut ::core::ffi::c_int;
    pub type mach_msg_type_number_t = natural_t;
    pub type kern_return_t = ::core::ffi::c_int;
    pub type vm_region_extended_info_data_t = vm_region_extended_info;
    pub type vm_prot_t = ::core::ffi::c_int;
    pub type task_name_t = mach_port_t;
    pub type task_flavor_t = natural_t;
    pub type task_info_t = *mut integer_t;

    pub const VM_PROT_READ: vm_prot_t = 1;
    pub const VM_PROT_WRITE: vm_prot_t = 1 << 1;
    pub const VM_PROT_EXECUTE: vm_prot_t = 1 << 2;

    // https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/bsd/sys/proc_info.h#L826
    pub const PROC_PIDPATHINFO_MAXSIZE: u32 = 4096;

    pub const VM_REGION_EXTENDED_INFO: vm_region_flavor_t = 13;
    pub const KERN_SUCCESS: kern_return_t = 0;
    pub const MACH_PORT_NULL: mach_port_t = 0;
    pub const TASK_DYLD_INFO: ::core::ffi::c_uint = 17;

    #[repr(C)]
    #[derive(Debug)]
    pub struct vm_region_extended_info {
        pub protection: vm_prot_t,
        pub user_tag: ::core::ffi::c_uint,
        pub pages_resident: ::core::ffi::c_uint,
        pub pages_shared_now_private: ::core::ffi::c_uint,
        pub pages_swapped_out: ::core::ffi::c_uint,
        pub pages_dirtied: ::core::ffi::c_uint,
        pub ref_count: ::core::ffi::c_uint,
        pub shadow_depth: ::core::ffi::c_ushort,
        pub external_pager: ::core::ffi::c_uchar,
        pub share_mode: ::core::ffi::c_uchar,
        pub pages_reusable: ::core::ffi::c_uint,
    }

    #[repr(C, packed(4))]
    #[derive(Copy, Clone, Debug, Default, Hash, PartialOrd, PartialEq, Eq, Ord)]
    pub struct task_dyld_info {
        pub all_image_info_addr: mach_vm_address_t,
        pub all_image_info_size: mach_vm_size_t,
        pub all_image_info_format: integer_t,
    }

    extern "C" {
        static mach_task_self_: mach_port_t;
        pub fn task_for_pid(
            target_tport: mach_port_name_t,
            pid: ::core::ffi::c_int,
            tn: *mut mach_port_name_t,
        ) -> kern_return_t;
        pub fn proc_regionfilename(
            pid: ::core::ffi::c_int,
            address: u64,
            buffer: *mut ::core::ffi::c_void,
            buffersize: u32,
        ) -> ::core::ffi::c_int;
        pub fn mach_vm_region(
            target_task: vm_task_entry_t,
            address: *mut mach_vm_address_t,
            size: *mut mach_vm_size_t,
            flavor: vm_region_flavor_t,
            info: vm_region_info_t,
            infoCnt: *mut mach_msg_type_number_t,
            object_name: *mut mach_port_t,
        ) -> kern_return_t;
        pub fn task_info(
            target_task: task_name_t,
            flavor: task_flavor_t,
            task_info_out: task_info_t,
            task_info_outCnt: *mut mach_msg_type_number_t,
        ) -> kern_return_t;
    }

    #[allow(clippy::missing_safety_doc)] // FIXME
    pub unsafe fn mach_task_self() -> mach_port_t {
        mach_task_self_
    }
}

use core::mem;

pub fn task_for_pid(pid: i32) -> std::io::Result<ffi::mach_port_name_t> {
    let mut task: ffi::mach_port_name_t = ffi::MACH_PORT_NULL;
    let result = unsafe { ffi::task_for_pid(ffi::mach_task_self(), pid, &mut task) };
    if result != ffi::KERN_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(result));
    }

    Ok(task)
}

pub fn proc_regionfilename(pid: i32, address: u64) -> Option<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity((ffi::PROC_PIDPATHINFO_MAXSIZE - 1) as _);
    let ptr = buf.as_mut_ptr() as *mut ::core::ffi::c_void;
    let size = buf.capacity() as u32;

    let result = unsafe { ffi::proc_regionfilename(pid, address, ptr, size) };

    if result <= 0 {
        None
    } else {
        unsafe {
            buf.set_len(result as _);
        }
        Some(buf)
    }
}

pub fn task_info(task: ffi::mach_port_name_t) -> Option<ffi::task_dyld_info> {
    const TASK_DYLD_INFO_COUNT: usize =
        mem::size_of::<ffi::task_dyld_info>() / mem::size_of::<ffi::natural_t>();
    let mut count = TASK_DYLD_INFO_COUNT;
    let mut dyld_info = unsafe { mem::zeroed::<ffi::task_dyld_info>() };
    let result = unsafe {
        ffi::task_info(
            task,
            ffi::TASK_DYLD_INFO,
            &mut dyld_info as *mut ffi::task_dyld_info as ffi::task_info_t,
            &mut count as *mut usize as *mut ffi::mach_msg_type_number_t,
        )
    };

    if result != ffi::KERN_SUCCESS {
        None
    } else {
        Some(dyld_info)
    }
}

// todo 还有一大堆
// https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/osfmk/mach/vm_statistics.h#L489
#[derive(Debug)]
pub enum VmTag {
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
    MallocNano,
    Dylib,
    Dyld,
    DyldMalloc,
    Other(u32),
}

impl From<u32> for VmTag {
    fn from(user_tag: u32) -> Self {
        match user_tag {
            1 => VmTag::Malloc,
            2 => VmTag::MallocSmall,
            3 => VmTag::MallocLarge,
            4 => VmTag::MallocHuge,
            5 => VmTag::Sbrk,
            6 => VmTag::Realloc,
            7 => VmTag::MallocTiny,
            8 => VmTag::MallocLargeReusable,
            9 => VmTag::MallocLargeReused,
            11 => VmTag::MallocNano,
            30 => VmTag::Stack,
            33 => VmTag::Dylib,
            60 => VmTag::Dyld,
            61 => VmTag::DyldMalloc,
            tag => VmTag::Other(tag),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Map {
    addr: ffi::mach_vm_address_t,
    size: ffi::mach_vm_size_t,
    count: ffi::mach_msg_type_number_t,
    info: ffi::vm_region_extended_info,
}

impl crate::MapExt for Map {
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
    pub fn tag(&self) -> VmTag {
        VmTag::from(self.info.user_tag)
    }

    fn end(&self) -> u64 {
        self.addr + self.size
    }
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

impl Iterator for MapIter {
    type Item = Map;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = mem::size_of::<ffi::vm_region_extended_info_data_t>() as ffi::mach_msg_type_number_t;
        let mut object_name: ffi::mach_port_t = 0;
        let mut size = unsafe { mem::zeroed::<ffi::mach_vm_size_t>() };
        let mut info = unsafe { mem::zeroed::<ffi::vm_region_extended_info_data_t>() };
        let result = unsafe {
            ffi::mach_vm_region(
                self.task,
                &mut self.addr,
                &mut size,
                ffi::VM_REGION_EXTENDED_INFO,
                &mut info as *mut ffi::vm_region_extended_info_data_t as ffi::vm_region_info_t,
                &mut count,
                &mut object_name,
            )
        };
        if result != ffi::KERN_SUCCESS {
            return None;
        }
        let region = Map { addr: self.addr, size, count, info };
        self.addr = region.end();
        Some(region)
    }
}
