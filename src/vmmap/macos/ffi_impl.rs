use core::mem;

use super::ffi;

use super::ffi::{
    dyld_all_image_infos, dyld_image_info, kern_return_t, mach_header_64, mach_vm_address_t, mach_vm_size_t,
    segment_command_64,
};

// macos 64 bit only
pub fn mach_vm_read_overwrite(
    target_task: ffi::vm_task_entry_t,
    address: ffi::mach_vm_address_t,
    size: ffi::mach_vm_size_t,
    data: ffi::mach_vm_address_t,
    outsize: *mut ffi::mach_vm_size_t,
) -> Result<(), ffi::kern_return_t> {
    let result: ffi::kern_return_t =
        unsafe { ffi::mach_vm_read_overwrite(target_task, address, size, data, outsize) };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub fn mach_vm_write(
    target_task: ffi::vm_map_t,
    address: mach_vm_address_t,
    buf: &[u8],
) -> Result<(), ffi::kern_return_t> {
    let result = unsafe { ffi::mach_vm_write(target_task, address, buf.as_ptr() as _, buf.len() as _) };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }

    Ok(())
}

pub fn task_for_pid(pid: i32) -> Result<ffi::mach_port_name_t, ffi::kern_return_t> {
    let mut task: ffi::mach_port_name_t = ffi::MACH_PORT_NULL;
    let result = unsafe { ffi::task_for_pid(ffi::mach_task_self_, pid, &mut task) };
    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }

    Ok(task)
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

#[allow(dead_code)]
pub fn get_offset(task: ffi::mach_port_name_t) -> Result<u64, kern_return_t> {
    const TASK_DYLD_INFO_COUNT: ffi::mach_msg_type_number_t = (mem::size_of::<ffi::task_dyld_info>()
        / mem::size_of::<ffi::natural_t>())
        as ffi::mach_msg_type_number_t;

    let mut count = TASK_DYLD_INFO_COUNT;
    let mut dyld_info = unsafe { mem::zeroed::<ffi::task_dyld_info>() };
    let result: ffi::kern_return_t = unsafe {
        ffi::task_info(task, ffi::TASK_DYLD_INFO, &mut dyld_info as *mut ffi::task_dyld_info as _, &mut count)
    };

    if result != ffi::KERN_SUCCESS {
        return Err(result);
    }

    let mut image_infos = unsafe { mem::zeroed::<ffi::dyld_all_image_infos>() };
    let mut read_len = mem::size_of_val(&image_infos) as mach_vm_size_t;

    mach_vm_read_overwrite(
        task,
        dyld_info.all_image_info_addr,
        read_len,
        &mut image_infos as *mut dyld_all_image_infos as _,
        &mut read_len,
    )?;

    // 只需要第一个
    let mut module = unsafe { mem::zeroed::<ffi::dyld_image_info>() };
    let mut read_len = (mem::size_of::<dyld_image_info>()) as mach_vm_size_t;
    mach_vm_read_overwrite(
        task,
        image_infos.infoArray as mach_vm_address_t,
        read_len,
        &mut module as *mut ffi::dyld_image_info as _,
        &mut read_len,
    )?;

    let mut header = unsafe { mem::zeroed::<ffi::mach_header_64>() };
    let mut read_len = mem::size_of_val(&header) as mach_vm_size_t;
    mach_vm_read_overwrite(
        task,
        module.imageLoadAddress as u64,
        read_len,
        &mut header as *mut mach_header_64 as mach_vm_address_t,
        &mut read_len,
    )?;

    let mut commands_buffer = vec![0_i8; header.sizeofcmds as usize];
    let mut read_len = mach_vm_size_t::from(header.sizeofcmds);
    mach_vm_read_overwrite(
        task,
        (module.imageLoadAddress as usize + mem::size_of_val(&header)) as _,
        read_len,
        commands_buffer.as_mut_ptr() as mach_vm_address_t,
        &mut read_len,
    )?;

    let command = unsafe { *(commands_buffer.as_ptr() as *const segment_command_64) };
    let offset = module.imageLoadAddress as u64
        - unsafe { *(commands_buffer.as_ptr().offset(command.cmdsize as _) as *const segment_command_64) }
            .vmaddr
        + command.vmaddr;

    Ok(offset)
}

#[allow(unused)]
pub struct MapRange {
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
    pub const fn new(task: ffi::mach_port_name_t) -> Self {
        Self { task, addr: 1 }
    }
}

impl Default for MapIter {
    fn default() -> Self {
        Self { task: unsafe { ffi::mach_task_self_ }, addr: 1 }
    }
}

// TODO Maybe shouldnot use iterator?
impl Iterator for MapIter {
    type Item = MapRange;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = mem::size_of::<ffi::vm_region_extended_info_data_t>() as ffi::mach_msg_type_number_t;
        let mut object_name: ffi::mach_port_t = 0;
        let mut size = unsafe { mem::zeroed::<ffi::mach_vm_size_t>() };
        let mut info = unsafe { mem::zeroed::<ffi::vm_region_extended_info_data_t>() };
        let result: ffi::kern_return_t = mach_vm_region(
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
        let region = MapRange { addr: self.addr, size, count, info };
        self.addr += region.size;
        Some(region)
    }
}

pub fn proc_regionfilename(pid: i32, address: u64) -> Result<String, ffi::kern_return_t> {
    let mut buf: Vec<u8> = Vec::with_capacity((ffi::PROC_PIDPATHINFO_MAXSIZE - 1) as _);
    let ptr = buf.as_mut_ptr() as *mut ::core::ffi::c_void;
    let size = buf.capacity() as u32;

    let result: ffi::kern_return_t = unsafe { ffi::proc_regionfilename(pid, address, ptr, size) };

    if result <= 0 {
        Err(result)
    } else {
        unsafe {
            buf.set_len(result as _);
        }
        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

pub fn proc_pidpath(pid: i32) -> Result<String, ffi::kern_return_t> {
    let pathbuf = [0u8; 4096];
    unsafe {
        let ret = ffi::proc_pidpath(pid, mem::transmute(pathbuf.as_ptr()), pathbuf.len() as _);
        if ret < 0 {
            return Err(ret);
        }

        Ok(std::ffi::CStr::from_ptr(mem::transmute(pathbuf.as_ptr()))
            .to_string_lossy()
            .to_string())
    }
}
