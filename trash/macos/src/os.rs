// safe api

use core::mem;

use super::ffi;

// macos 64 bit only
pub fn mach_vm_read_overwrite(
    task: ffi::vm_task_entry_t,
    addr: u64,
    buf: &mut [u8],
) -> Result<(), ffi::kern_return_t> {
    let mut read_len: u64 = 0;
    let result: ffi::kern_return_t = unsafe {
        ffi::mach_vm_read_overwrite(task, addr, buf.len() as _, buf.as_mut_ptr() as _, &mut read_len)
    };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub fn mach_vm_write(task: ffi::vm_task_entry_t, addr: u64, buf: &[u8]) -> Result<(), ffi::kern_return_t> {
    let result: ffi::kern_return_t =
        unsafe { ffi::mach_vm_write(task, addr, buf.as_ptr() as _, buf.len() as _) };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub fn task_for_pid(pid: i32) -> Result<ffi::mach_port_name_t, ffi::kern_return_t> {
    let mut task: ffi::mach_port_name_t = ffi::MACH_PORT_NULL;
    let result = unsafe { ffi::task_for_pid(ffi::mach_task_self(), pid, &mut task) };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }

    Ok(task)
}

pub fn proc_regionfilename(pid: i32, address: u64) -> Option<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity((ffi::PROC_PIDPATHINFO_MAXSIZE - 1) as _);
    let ptr = buf.as_mut_ptr() as *mut ::core::ffi::c_void;
    let size = buf.capacity() as u32;

    let result: ffi::kern_return_t = unsafe { ffi::proc_regionfilename(pid, address, ptr, size) };

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
    let result: ffi::kern_return_t = unsafe {
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

pub fn mach_vm_region(
    target_task: ffi::vm_task_entry_t,
    address: &mut ffi::mach_vm_address_t,
    size: &mut ffi::mach_vm_size_t,
    flavor: ffi::vm_region_flavor_t,
    info: ffi::vm_region_info_t,
    info_cnt: &mut ffi::mach_msg_type_number_t,
    object_name: &mut ffi::mach_port_t,
) -> ffi::kern_return_t {
    unsafe { ffi::mach_vm_region(target_task, address, size, flavor, info, info_cnt, object_name) }
}
