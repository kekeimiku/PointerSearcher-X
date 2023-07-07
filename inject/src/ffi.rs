#![allow(non_snake_case, unused)]

use std::mem;

use super::{
    bindgen,
    bindgen::{
        kern_return_t, mach_msg_type_number_t, mach_port_name_t, mach_port_t, mach_task_self_, mach_vm_address_t,
        mach_vm_size_t, natural_t, task_dyld_info_data_t, task_flavor_t, task_info_t, task_name_t, vm_map_read_t,
        KERN_SUCCESS, MACH_PORT_NULL,
    },
};
use crate::bindgen::{
    arm_thread_state64_t, boolean_t, task_t, thread_act_t, thread_basic_info_data_t, thread_flavor_t, thread_info_t,
    thread_inspect_t, thread_read_t, thread_state_flavor_t, thread_state_t, vm_map_t, vm_offset_t, vm_prot_t,
};

pub const TASK_DYLD_INFO_COUNT: usize = mem::size_of::<task_dyld_info_data_t>() / mem::size_of::<natural_t>();

pub const ARM_THREAD_STATE64_COUNT: mach_msg_type_number_t =
    (mem::size_of::<arm_thread_state64_t>() / mem::size_of::<natural_t>()) as mach_msg_type_number_t;

pub const THREAD_BASIC_INFO_COUNT: mach_msg_type_number_t =
    (mem::size_of::<thread_basic_info_data_t>() / mem::size_of::<natural_t>()) as mach_msg_type_number_t;

#[allow(clippy::missing_safety_doc)] // FIXME
unsafe fn mach_task_self() -> mach_port_t {
    mach_task_self_
}

pub const VM_PROT_NONE: vm_prot_t = 0;
pub const VM_PROT_READ: vm_prot_t = 1;
pub const VM_PROT_WRITE: vm_prot_t = 1 << 1;
pub const VM_PROT_EXECUTE: vm_prot_t = 1 << 2;
pub const VM_PROT_NO_CHANGE: vm_prot_t = 1 << 3;
pub const VM_PROT_COPY: vm_prot_t = 1 << 4;
pub const VM_PROT_WANTS_COPY: vm_prot_t = 1 << 4;
pub const VM_PROT_DEFAULT: vm_prot_t = VM_PROT_READ | VM_PROT_WRITE;
pub const VM_PROT_ALL: vm_prot_t = VM_PROT_READ | VM_PROT_WRITE | VM_PROT_EXECUTE;

pub unsafe fn task_for_pid(pid: i32) -> Result<mach_port_name_t, kern_return_t> {
    let mut task: mach_port_name_t = MACH_PORT_NULL;
    let result = unsafe { bindgen::task_for_pid(mach_task_self(), pid, &mut task) };
    if result != KERN_SUCCESS {
        return Err(result);
    }
    Ok(task)
}

pub unsafe fn mach_vm_read_overwrite(
    target_task: vm_map_read_t,
    address: mach_vm_address_t,
    size: mach_vm_size_t,
    data: mach_vm_address_t,
    outsize: *mut mach_vm_size_t,
) -> Result<(), kern_return_t> {
    let result = bindgen::mach_vm_read_overwrite(target_task, address, size, data, outsize);
    if result != KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub unsafe fn task_info(
    target_task: task_name_t,
    flavor: task_flavor_t,
    task_info_out: task_info_t,
    task_info_outCnt: *mut mach_msg_type_number_t,
) -> Result<(), kern_return_t> {
    let ret = bindgen::task_info(target_task, flavor, task_info_out, task_info_outCnt);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

pub unsafe fn mach_vm_allocate(
    target: vm_map_t,
    address: *mut mach_vm_address_t,
    size: mach_vm_size_t,
    flags: ::core::ffi::c_int,
) -> Result<(), kern_return_t> {
    let ret = bindgen::mach_vm_allocate(target, address, size, flags);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

pub unsafe fn mach_vm_write(
    target_task: vm_map_t,
    address: mach_vm_address_t,
    data: vm_offset_t,
    dataCnt: mach_msg_type_number_t,
) -> Result<(), kern_return_t> {
    let ret = bindgen::mach_vm_write(target_task, address, data, dataCnt);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

pub unsafe fn mach_vm_protect(
    target_task: vm_map_t,
    address: mach_vm_address_t,
    size: mach_vm_size_t,
    set_maximum: boolean_t,
    new_protection: vm_prot_t,
) -> Result<(), kern_return_t> {
    let ret = bindgen::mach_vm_protect(target_task, address, size, set_maximum, new_protection);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

pub unsafe fn thread_create_running(
    parent_task: task_t,
    flavor: thread_state_flavor_t,
    new_state: thread_state_t,
    new_stateCnt: mach_msg_type_number_t,
    child_act: *mut thread_act_t,
) -> Result<(), kern_return_t> {
    let ret = bindgen::thread_create_running(parent_task, flavor, new_state, new_stateCnt, child_act);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}
