#![allow(non_snake_case, unused)]

use std::{ffi::CStr, mem};

use machx::{
    boolean::boolean_t,
    error::mach_error_t,
    kern_return::{kern_return_t, KERN_SUCCESS},
    mach_types::{task_name_t, task_t, thread_act_t},
    message::mach_msg_type_number_t,
    port::{mach_port_name_t, mach_port_t, MACH_PORT_NULL},
    structs::arm_thread_state64_t,
    task_info::{task_dyld_info_data_t, task_flavor_t, task_info_t},
    thread_status::{thread_state_flavor_t, thread_state_t},
    vm::thread_basic_info_data_t,
    vm_prot::vm_prot_t,
    vm_types::{mach_vm_address_t, mach_vm_size_t, natural_t, vm_map_t, vm_offset_t},
};

pub const TASK_DYLD_INFO_COUNT: usize = mem::size_of::<task_dyld_info_data_t>() / mem::size_of::<natural_t>();

pub const ARM_THREAD_STATE64_COUNT: mach_msg_type_number_t =
    (mem::size_of::<arm_thread_state64_t>() / mem::size_of::<natural_t>()) as mach_msg_type_number_t;

pub const THREAD_BASIC_INFO_COUNT: mach_msg_type_number_t =
    (mem::size_of::<thread_basic_info_data_t>() / mem::size_of::<natural_t>()) as mach_msg_type_number_t;

#[inline]
pub unsafe fn mach_error(error_value: mach_error_t) -> String {
    let ptr = machx::error::mach_error_string(error_value);
    String::from(std::str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()))
}

#[inline]
pub unsafe fn task_for_pid(pid: i32) -> Result<mach_port_name_t, kern_return_t> {
    let mut task: mach_port_name_t = MACH_PORT_NULL;
    let result = unsafe { machx::traps::task_for_pid(machx::traps::mach_task_self(), pid, &mut task) };
    if result != KERN_SUCCESS {
        return Err(result);
    }
    Ok(task)
}

#[inline]
pub unsafe fn mach_vm_read_overwrite(
    target_task: mach_port_t,
    address: mach_vm_address_t,
    size: mach_vm_size_t,
    data: mach_vm_address_t,
    outsize: *mut mach_vm_size_t,
) -> Result<(), kern_return_t> {
    let result = machx::vm::mach_vm_read_overwrite(target_task, address, size, data, outsize);
    if result != KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

#[inline]
pub unsafe fn task_info(
    target_task: task_name_t,
    flavor: task_flavor_t,
    task_info_out: task_info_t,
    task_info_outCnt: *mut mach_msg_type_number_t,
) -> Result<(), kern_return_t> {
    let ret = machx::task::task_info(target_task, flavor, task_info_out, task_info_outCnt);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

#[inline]
pub unsafe fn mach_vm_allocate(
    target: vm_map_t,
    address: *mut mach_vm_address_t,
    size: mach_vm_size_t,
    flags: ::core::ffi::c_int,
) -> Result<(), kern_return_t> {
    let ret = machx::vm::mach_vm_allocate(target, address, size, flags);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

#[inline]
pub unsafe fn mach_vm_write(
    target_task: vm_map_t,
    address: mach_vm_address_t,
    data: vm_offset_t,
    dataCnt: mach_msg_type_number_t,
) -> Result<(), kern_return_t> {
    let ret = machx::vm::mach_vm_write(target_task, address, data, dataCnt);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

#[inline]
pub unsafe fn mach_vm_protect(
    target_task: vm_map_t,
    address: mach_vm_address_t,
    size: mach_vm_size_t,
    set_maximum: boolean_t,
    new_protection: vm_prot_t,
) -> Result<(), kern_return_t> {
    let ret = machx::vm::mach_vm_protect(target_task, address, size, set_maximum, new_protection);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}

#[inline]
pub unsafe fn thread_create_running(
    parent_task: task_t,
    flavor: thread_state_flavor_t,
    new_state: thread_state_t,
    new_stateCnt: mach_msg_type_number_t,
    child_act: *mut thread_act_t,
) -> Result<(), kern_return_t> {
    let ret = machx::task::thread_create_running(parent_task, flavor, new_state, new_stateCnt, child_act);
    if ret != KERN_SUCCESS {
        return Err(ret);
    }
    Ok(())
}
