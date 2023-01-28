#![allow(non_camel_case_types)]

pub type mach_port_t = ::core::ffi::c_uint;
pub type natural_t = ::core::ffi::c_uint;
pub type mach_port_name_t = natural_t;
pub type kern_return_t = ::core::ffi::c_int;
pub type vm_task_entry_t = mach_port_t;
pub type mach_vm_address_t = u64;
pub type mach_vm_size_t = u64;
pub type vm_region_flavor_t = ::core::ffi::c_int;
pub type vm_region_info_t = *mut ::core::ffi::c_int;
pub type mach_msg_type_number_t = natural_t;
pub type vm_region_basic_info_data_64_t = vm_region_basic_info_64;
pub type vm_prot_t = ::core::ffi::c_int;
pub type vm_inherit_t = ::core::ffi::c_uint;
pub type memory_object_offset_t = ::core::ffi::c_ulonglong;
pub type vm_behavior_t = ::core::ffi::c_int;

pub const PROC_PIDPATHINFO_MAXSIZE: u32 = 4096;
pub const VM_PROT_READ: vm_prot_t = 1;
pub const VM_PROT_WRITE: vm_prot_t = 1 << 1;
pub const VM_PROT_EXECUTE: vm_prot_t = 1 << 2;

#[cfg(target_arch = "x86_64")]
pub type boolean_t = ::core::ffi::c_uint;

#[cfg(target_arch = "aarch64")]
pub type boolean_t = ::core::ffi::c_int;

#[repr(C, packed(4))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct vm_region_basic_info_64 {
    pub protection: vm_prot_t,
    pub max_protection: vm_prot_t,
    pub inheritance: vm_inherit_t,
    pub shared: boolean_t,
    pub reserved: boolean_t,
    pub offset: memory_object_offset_t,
    pub behavior: vm_behavior_t,
    pub user_wired_count: ::core::ffi::c_ushort,
}

pub type pid_t = i32;

pub const MACH_PORT_NULL: mach_port_t = 0;
pub const KERN_SUCCESS: kern_return_t = 0;
pub const VM_REGION_BASIC_INFO_64: vm_region_flavor_t = 9;

extern "C" {
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
    static mach_task_self_: mach_port_t;
    pub fn task_for_pid(
        target_tport: mach_port_name_t,
        pid: pid_t,
        tn: *mut mach_port_name_t,
    ) -> kern_return_t;
    pub fn vm_read_overwrite(
        target_task: mach_port_t,
        address: mach_vm_address_t,
        size: mach_vm_size_t,
        data: mach_vm_address_t,
        out_size: *mut mach_vm_size_t,
    ) -> kern_return_t;
    fn mach_vm_write(
        target_task: vm_map_t,
        address: mach_vm_address_t,
        data: vm_offset_t,
        dataCnt: mach_msg_type_number_t,
    ) -> kern_return_t;
}

pub unsafe fn mach_task_self() -> mach_port_t {
    mach_task_self_
}
