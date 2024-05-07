#![allow(clippy::missing_safety_doc)]

use core::{ffi::CStr, fmt::Display, mem, ops::Range, str};
use std::{
    io,
    io::{Error, ErrorKind},
};

use machx::{
    dyld_images::{dyld_all_image_infos, dyld_image_info, mach_header_64, segment_command_64},
    error::mach_error_string,
    kern_return::{kern_return_t, KERN_SUCCESS},
    libproc::{
        proc_pidinfo, proc_regionwithpathinfo, PROC_PIDREGIONPATHINFO, PROC_PIDREGIONPATHINFO_SIZE,
    },
    loader::{LC_SEGMENT_64, SEG_TEXT},
    mach_types::task_name_t,
    task::task_info,
    task_info::{task_dyld_info, task_info_t, TASK_DYLD_INFO},
    vm::mach_vm_read_overwrite,
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::ModuleMap;

#[derive(Debug)]
pub enum QueryProcError {
    PidInfo(io::Error),
    TaskInfo(kern_return_t),
    ReadMem(kern_return_t),
    Utf8(str::Utf8Error),
}

impl From<str::Utf8Error> for QueryProcError {
    fn from(value: str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl Display for QueryProcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryProcError::PidInfo(err) => write!(f, "{err}"),
            QueryProcError::TaskInfo(err) => {
                write!(
                    f,
                    "code: {err}, msg: {}",
                    unsafe { CStr::from_ptr(mach_error_string(*err)) }.to_string_lossy()
                )
            }
            QueryProcError::ReadMem(err) => {
                write!(
                    f,
                    "code: {err}, msg: {}",
                    unsafe { CStr::from_ptr(mach_error_string(*err)) }.to_string_lossy()
                )
            }
            QueryProcError::Utf8(err) => write!(f, "{err}"),
        }
    }
}

pub unsafe fn list_unknown_maps(pid: i32) -> Result<Vec<Range<usize>>, QueryProcError> {
    let mut unknown_maps = Vec::with_capacity(128);
    let mut address = 0;
    let mut last_ino = 0;
    loop {
        let mut rwpi = mem::zeroed::<proc_regionwithpathinfo>();
        let written = proc_pidinfo(
            pid,
            PROC_PIDREGIONPATHINFO,
            address,
            &mut rwpi as *mut proc_regionwithpathinfo as _,
            PROC_PIDREGIONPATHINFO_SIZE,
        );

        if written <= 0 {
            let err = Error::last_os_error();
            if err.raw_os_error() == Some(3) || err.raw_os_error() == Some(22) {
                break;
            }
            return Err(QueryProcError::PidInfo(err));
        }
        if written < PROC_PIDREGIONPATHINFO_SIZE {
            let err = Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "only recieved {}/{} bytes of struct",
                    written, PROC_PIDREGIONPATHINFO_SIZE
                ),
            );
            return Err(QueryProcError::PidInfo(err));
        }
        address = rwpi.prp_prinfo.pri_address + rwpi.prp_prinfo.pri_size;
        let ino = rwpi.prp_vip.vip_vi.vi_stat.vst_ino;
        let path = CStr::from_ptr(rwpi.prp_vip.vip_path.as_ptr());
        if ino == last_ino
            && path.is_empty()
            && rwpi.prp_prinfo.pri_protection & 1 != 0
            && rwpi.prp_prinfo.pri_protection & 2 != 0
            && rwpi.prp_prinfo.pri_share_mode != 3
        {
            let range = rwpi.prp_prinfo.pri_address as usize
                ..(rwpi.prp_prinfo.pri_size + rwpi.prp_prinfo.pri_address) as usize;
            unknown_maps.push(range);
            last_ino = ino
        }
    }
    Ok(unknown_maps)
}

// 获取进程自身加载的模块 但是忽略共享缓存之类。
pub unsafe fn list_image_maps(
    pid: i32,
    task: task_name_t,
) -> Result<ModuleMap<usize, String>, QueryProcError> {
    let mut dyld_info = mem::zeroed::<task_dyld_info>();
    let mut count = task_dyld_info::count() as u32;

    let kr = task_info(
        task,
        TASK_DYLD_INFO,
        &mut dyld_info as *mut task_dyld_info as task_info_t,
        &mut count,
    );
    if kr != KERN_SUCCESS {
        return Err(QueryProcError::TaskInfo(kr));
    }

    let mut image_infos = mem::zeroed::<dyld_all_image_infos>();
    let mut read_len = mem::size_of::<dyld_all_image_infos>() as u64;
    let kr = mach_vm_read_overwrite(
        task,
        dyld_info.all_image_info_addr,
        read_len,
        &mut image_infos as *mut dyld_all_image_infos as mach_vm_address_t,
        &mut read_len,
    );
    if kr != KERN_SUCCESS {
        return Err(QueryProcError::ReadMem(kr));
    }

    let mut modules = vec![mem::zeroed::<dyld_image_info>(); image_infos.infoArrayCount as usize];
    let mut read_len =
        (mem::size_of::<dyld_image_info>() * image_infos.infoArrayCount as usize) as u64;
    let kr = mach_vm_read_overwrite(
        task,
        image_infos.infoArray as mach_vm_address_t,
        read_len,
        modules.as_mut_ptr() as mach_vm_address_t,
        &mut read_len,
    );
    if kr != KERN_SUCCESS {
        return Err(QueryProcError::ReadMem(kr));
    }

    let mut module_maps = ModuleMap::new();

    for module in modules {
        // 也许有办法不必解析 mach-o ...
        let mut header = mem::zeroed::<mach_header_64>();
        let mut read_len = mem::size_of::<mach_header_64>() as u64;
        let kr = mach_vm_read_overwrite(
            task,
            module.imageLoadAddress as u64,
            read_len,
            &mut header as *mut mach_header_64 as mach_vm_address_t,
            &mut read_len,
        );
        if kr != KERN_SUCCESS {
            return Err(QueryProcError::ReadMem(kr));
        }

        let mut commands_buffer = vec![0_i8; header.sizeofcmds as usize];
        let mut read_len = mach_vm_size_t::from(header.sizeofcmds);
        let kr = mach_vm_read_overwrite(
            task,
            (module.imageLoadAddress as usize + mem::size_of_val(&header)) as mach_vm_size_t,
            read_len,
            commands_buffer.as_mut_ptr() as mach_vm_address_t,
            &mut read_len,
        );
        if kr != KERN_SUCCESS {
            return Err(QueryProcError::ReadMem(kr));
        }

        let mut offset: u32 = 0;
        let mut slide: u64 = 0;
        for _ in 0..header.ncmds {
            let command =
                *(commands_buffer.as_ptr().offset(offset as isize) as *const segment_command_64);
            if command.cmd == LC_SEGMENT_64 && command.segname[0..7].eq(SEG_TEXT) {
                slide = module.imageLoadAddress as u64 - command.vmaddr;
                break;
            }
            offset += command.cmdsize;
        }

        let mut image_end_address = 0;

        let mut offset = 0;
        for _ in 0..header.ncmds {
            let mut command =
                *(commands_buffer.as_ptr().offset(offset as isize) as *const segment_command_64);

            if command.cmd == LC_SEGMENT_64 {
                command.vmaddr += slide;
                image_end_address = command.vmaddr + command.vmsize;
            }
            offset += command.cmdsize;
        }

        let mut rwpi = mem::zeroed::<proc_regionwithpathinfo>();
        let written = proc_pidinfo(
            pid,
            PROC_PIDREGIONPATHINFO,
            module.imageLoadAddress as u64,
            &mut rwpi as *mut proc_regionwithpathinfo as _,
            PROC_PIDREGIONPATHINFO_SIZE,
        );

        if written <= 0 {
            let err = Error::last_os_error();
            if err.raw_os_error() == Some(3) || err.raw_os_error() == Some(22) {
                break;
            }
            return Err(QueryProcError::PidInfo(err));
        }

        if written < PROC_PIDREGIONPATHINFO_SIZE {
            let err = Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "only recieved {}/{} bytes of struct",
                    written, PROC_PIDREGIONPATHINFO_SIZE
                ),
            );
            return Err(QueryProcError::PidInfo(err));
        }

        if rwpi.prp_vip.vip_vi.vi_stat.vst_dev != 0 && rwpi.prp_vip.vip_vi.vi_stat.vst_ino != 0 {
            let mut read_len = 512_u64;
            let mut image_filename = [0_i8; 512];
            let kr = mach_vm_read_overwrite(
                task,
                module.imageFilePath as mach_vm_address_t,
                read_len,
                image_filename.as_mut_ptr() as mach_vm_address_t,
                &mut read_len,
            );
            if kr != KERN_SUCCESS {
                return Err(QueryProcError::ReadMem(kr));
            }
            let cstr = CStr::from_ptr(image_filename.as_ptr());
            let pathname = cstr.to_str()?.to_string();
            let range = module.imageLoadAddress as usize..image_end_address as usize;
            module_maps.insert(range, pathname);
        }
    }

    Ok(module_maps)
}
