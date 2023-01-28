#![allow(non_camel_case_types, unused)]
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
pub type vm_map_t = mach_port_t;
pub type vm_offset_t = usize;

pub const VM_PROT_READ: vm_prot_t = 1;
pub const VM_PROT_WRITE: vm_prot_t = 1 << 1;
pub const VM_PROT_EXECUTE: vm_prot_t = 1 << 2;

// https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/bsd/sys/proc_info.h#L826
pub const PROC_PIDPATHINFO_MAXSIZE: u32 = 4096;

pub const VM_REGION_EXTENDED_INFO: vm_region_flavor_t = 13;
pub const MACH_PORT_NULL: mach_port_t = 0;
pub const TASK_DYLD_INFO: ::core::ffi::c_uint = 17;

// TODO enum all error code
// https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/osfmk/mach/kern_return.h#L72
pub const KERN_SUCCESS: kern_return_t = 0;

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
        info_cnt: *mut mach_msg_type_number_t,
        object_name: *mut mach_port_t,
    ) -> kern_return_t;
    pub fn task_info(
        target_task: task_name_t,
        flavor: task_flavor_t,
        task_info_out: task_info_t,
        task_info_outCnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn mach_vm_read_overwrite(
        target_task: vm_task_entry_t,
        address: mach_vm_address_t,
        size: mach_vm_size_t,
        data: mach_vm_address_t,
        outsize: *mut mach_vm_size_t,
    ) -> kern_return_t;
    pub fn mach_vm_write(
        target_task: vm_map_t,
        address: mach_vm_address_t,
        data: vm_offset_t,
        dataCnt: mach_msg_type_number_t,
    ) -> kern_return_t;
}

#[allow(clippy::missing_safety_doc)] // FIXME
pub unsafe fn mach_task_self() -> mach_port_t {
    mach_task_self_
}
