use std::{ffi::CStr, mem, path::PathBuf};

use mach2::{
    dyld_images::{dyld_all_image_infos, dyld_image_info, mach_header_64, segment_command_64},
    kern_return::{kern_return_t, KERN_SUCCESS},
    message::mach_msg_type_number_t,
    port::mach_port_t,
    task::task_info,
    task_info::{task_dyld_info, task_info_t, TASK_DYLD_INFO},
    vm,
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::vmmap64::Process;

const TASK_DYLD_INFO_COUNT: mach_msg_type_number_t =
    (mem::size_of::<task_dyld_info>() / mem::size_of::<mach2::vm_types::natural_t>()) as mach_msg_type_number_t;

impl Process {
    pub fn get_dyld_infos(&self) -> Result<impl Iterator<Item = Result<DyldInfo, kern_return_t>>, kern_return_t> {
        unsafe {
            let infos = get_dyld_image_infos(self.task)?;
            Ok(TryDyldInfoIter::new(self.task, infos.into_iter()))
        }
    }
}

pub struct DyldInfo {
    pub filename: PathBuf,
    pub address: usize,
    pub file_mod_date: usize,
    pub segment: segment_command_64,
}

pub unsafe fn mach_vm_read_overwrite(
    target_task: mach_port_t,
    address: mach_vm_address_t,
    size: mach_vm_size_t,
    data: mach_vm_address_t,
    outsize: *mut mach_vm_size_t,
) -> Result<(), kern_return_t> {
    let result = vm::mach_vm_read_overwrite(target_task, address, size, data, outsize);
    if result != KERN_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub unsafe fn get_dyld_image_infos(task: mach_port_t) -> Result<Vec<dyld_image_info>, kern_return_t> {
    let mut dyld_info = mem::zeroed::<task_dyld_info>();
    let mut count = TASK_DYLD_INFO_COUNT;
    let result = task_info(task, TASK_DYLD_INFO, &mut dyld_info as *mut task_dyld_info as task_info_t, &mut count);
    if result != KERN_SUCCESS {
        return Err(result);
    }

    let mut image_infos = mem::zeroed::<dyld_all_image_infos>();
    let mut read_len = mem::size_of_val(&image_infos) as mach_vm_size_t;

    mach_vm_read_overwrite(
        task,
        dyld_info.all_image_info_addr,
        read_len,
        (&mut image_infos) as *mut dyld_all_image_infos as mach_vm_address_t,
        &mut read_len,
    )?;

    let mut modules = vec![mem::zeroed::<dyld_image_info>(); image_infos.infoArrayCount as usize];
    let mut read_len = (mem::size_of::<dyld_image_info>() * image_infos.infoArrayCount as usize) as mach_vm_size_t;

    mach_vm_read_overwrite(
        task,
        image_infos.infoArray as mach_vm_address_t,
        read_len,
        modules.as_mut_ptr() as mach_vm_address_t,
        &mut read_len,
    )?;

    Ok(modules)
}

pub struct TryDyldInfoIter<I> {
    task: mach_port_t,
    infos: I,
    slide: u64,
}

impl<I> TryDyldInfoIter<I> {
    fn new(task: mach_port_t, infos: I) -> Self {
        Self { task, infos, slide: 0 }
    }
}

impl<T> Iterator for TryDyldInfoIter<T>
where
    T: Iterator<Item = dyld_image_info>,
{
    type Item = Result<DyldInfo, kern_return_t>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let module = self.infos.next()?;
            let mut read_len = 512_u64;
            let mut image_filename = [0_i8; 512];
            if let Err(e) = mach_vm_read_overwrite(
                self.task,
                module.imageFilePath as mach_vm_address_t,
                read_len,
                image_filename.as_mut_ptr() as mach_vm_address_t,
                &mut read_len,
            ) {
                return Some(Err(e));
            }

            let filename = std::str::from_utf8_unchecked(CStr::from_ptr(image_filename.as_ptr()).to_bytes());

            let mut header = mem::zeroed::<mach_header_64>();
            let mut read_len = std::mem::size_of_val(&header) as mach_vm_size_t;
            if let Err(e) = mach_vm_read_overwrite(
                self.task,
                module.imageLoadAddress as u64,
                read_len,
                (&mut header) as *mut mach_header_64 as mach_vm_address_t,
                &mut read_len,
            ) {
                return Some(Err(e));
            }

            let mut commands_buffer = vec![0_i8; header.sizeofcmds as usize];
            let mut read_len = mach_vm_size_t::from(header.sizeofcmds);
            if let Err(e) = mach_vm_read_overwrite(
                self.task,
                (module.imageLoadAddress as usize + mem::size_of_val(&header)) as mach_vm_size_t,
                read_len,
                commands_buffer.as_mut_ptr() as mach_vm_address_t,
                &mut read_len,
            ) {
                return Some(Err(e));
            }

            let mut offset: u32 = 0;
            for _ in 0..header.ncmds {
                let command = *(commands_buffer.as_ptr().offset(offset as isize) as *const segment_command_64);
                if command.cmd == 0x19 && command.segname[0..7] == [95, 95, 84, 69, 88, 84, 0] {
                    self.slide = module.imageLoadAddress as u64 - command.vmaddr;
                    break;
                }
                offset += command.cmdsize;
            }

            let mut offset: u32 = 0;
            for _ in 0..header.ncmds {
                let mut command = *(commands_buffer.as_ptr().offset(offset as isize) as *const segment_command_64);
                if command.cmd == 0x19 {
                    command.vmaddr += self.slide;
                    return Some(Ok(DyldInfo {
                        filename: PathBuf::from(filename),
                        address: module.imageLoadAddress as usize,
                        file_mod_date: module.imageFileModDate,
                        segment: command,
                    }));
                } else {
                    offset += command.cmdsize;
                }
            }
        }
        None
    }
}

// #[derive(Debug)]
// pub struct Symbol {
//     pub addr: u64,
//     pub name: String,
// }

// pub struct SymbolIter {
//     pub task: mach_port_t,
//     pub nsyms: u32,
//     pub sym_addr: u64,
//     pub strings: u64,
//     pub shared_image_cache_slide: u64,
// }

// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct nlist_64 {
//     pub n_un: nlist_64__bindgen_ty_1,
//     pub n_type: u8,
//     pub n_sect: u8,
//     pub n_desc: u16,
//     pub n_value: u64,
// }
// #[repr(C)]
// #[derive(Copy, Clone)]
// pub union nlist_64__bindgen_ty_1 {
//     pub n_strx: u32,
// }

// impl Iterator for SymbolIter {
//     type Item = Result<Symbol, kern_return_t>;

//     fn next(&mut self) -> Option<Self::Item> {
//         unsafe {
//             if self.nsyms == 0 {
//                 return None;
//             }

//             let mut sym = mem::zeroed::<nlist_64>();
//             let mut sym_size = mem::size_of_val(&sym) as u64;
//             if let Err(e) = mach_vm_read_overwrite(
//                 self.task,
//                 self.sym_addr,
//                 sym_size,
//                 &mut sym as *mut _ as mach_vm_address_t,
//                 &mut sym_size,
//             ) { return Some(Err(e));
//             }

//             let mut name = [0; 512];
//             let mut size = mem::size_of_val(&name) as u64;
//             let symname_offset = self.strings + sym.n_un.n_strx as u64;

//             if let Err(e) = mach_vm_read_overwrite(
//                 self.task,
//                 symname_offset,
//                 size,
//                 name.as_mut_ptr() as mach_vm_address_t,
//                 &mut size,
//             ) { return Some(Err(e));
//             }

//             let sym_name =
// std::str::from_utf8_unchecked(CStr::from_ptr(name.as_ptr()).to_bytes());

//             self.sym_addr += sym_size;
//             self.nsyms -= 1;

//             Some(Ok(Symbol {
//                 addr: sym.n_value + self.shared_image_cache_slide,
//                 name: String::from(sym_name),
//             }))
//         }
//     }
// }

// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct load_command {
//     pub cmd: u32,
//     pub cmdsize: u32,
// }

// pub const LC_SEGMENT: u32 = 1;
// pub const LC_SEGMENT_64: u32 = 25;
// pub const SEG_TEXT: &[u8; 7usize] = b"__TEXT\0";
// pub const SEG_LINKEDIT: &[u8; 11usize] = b"__LINKEDIT\0";
// pub const LC_SYMTAB: u32 = 2;
// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct symtab_command {
//     pub cmd: u32,
//     pub cmdsize: u32,
//     pub symoff: u32,
//     pub nsyms: u32,
//     pub stroff: u32,
//     pub strsize: u32,
// }

// pub unsafe fn symbol(
//     task: mach_port_t,
//     library_header_address: mach_vm_address_t,
// ) -> Result<SymbolIter, kern_return_t> { let mut info =
//   mem::zeroed::<task_dyld_info>(); let mut count = TASK_DYLD_INFO_COUNT; let
//   result = task_info(task, TASK_DYLD_INFO, &mut info as *mut task_dyld_info
//   as task_info_t, &mut count); if result != KERN_SUCCESS { return
//   Err(result); }

//     let mut header = mem::zeroed::<mach_header_64>();
//     let mut size: mach_vm_size_t = mem::size_of_val(&header) as u64;

//     mach_vm_read_overwrite(task, library_header_address, size, &mut header as
// *mut _ as mach_vm_address_t, &mut size)?;

//     let mut infos = mem::zeroed::<dyld_all_image_infos>();
//     let mut size = info.all_image_info_size;

//     mach_vm_read_overwrite(task, info.all_image_info_addr, size, &mut infos
// as *mut _ as mach_vm_address_t, &mut size)?;

//     let shared_image_cache_slide = infos.sharedCacheSlide as u64;

//     let mut command = mem::zeroed::<load_command>();
//     let mut size = mem::size_of_val(&command) as u64;

//     let mut seg_linkedit_addr: mach_vm_address_t = 0;
//     let mut seg_text_addr: mach_vm_address_t = 0;
//     let mut symtab_addr: mach_vm_address_t = 0;
//     let mut load_command_address = library_header_address +
// mem::size_of::<mach_header_64>() as u64;

//     for _ in 0..header.ncmds {
//         mach_vm_read_overwrite(
//             task,
//             load_command_address,
//             size,
//             &mut command as *mut _ as mach_vm_address_t,
//             &mut size,
//         )?;

//         if command.cmd == LC_SEGMENT | LC_SEGMENT_64 {
//             let mut name = [0; 512];
//             let mut size = mem::size_of_val(&name) as u64;

//             let segname_offset = mem::offset_of!(segment_command_64,
// segname);

//             mach_vm_read_overwrite(
//                 task,
//                 load_command_address + segname_offset as u64,
//                 size,
//                 name.as_mut_ptr() as mach_vm_address_t,
//                 &mut size,
//             )?;

//             let seg_name = CStr::from_ptr(name.as_ptr()).to_bytes_with_nul();

//             if seg_name == SEG_TEXT {
//                 seg_text_addr = load_command_address;
//             } else if seg_name == SEG_LINKEDIT {
//                 seg_linkedit_addr = load_command_address;
//             }
//         }

//         if command.cmd == LC_SYMTAB {
//             symtab_addr = load_command_address;
//         }

//         load_command_address += command.cmdsize as u64;
//     }

//     let mut seg_linkedit = mem::zeroed::<segment_command_64>();
//     let mut seg_text = mem::zeroed::<segment_command_64>();
//     let mut symtab = mem::zeroed::<symtab_command>();

//     let mut size = mem::size_of_val(&seg_linkedit) as u64;

//     mach_vm_read_overwrite(task, seg_linkedit_addr, size, &mut seg_linkedit
// as *mut _ as mach_vm_address_t, &mut size)?;

//     let mut size = mem::size_of_val(&seg_text) as u64;

//     mach_vm_read_overwrite(task, seg_text_addr, size, &mut seg_text as *mut _
// as mach_vm_address_t, &mut size)?;

//     let mut size = mem::size_of_val(&symtab) as u64;

//     mach_vm_read_overwrite(task, symtab_addr, size, &mut symtab as *mut _ as
// mach_vm_address_t, &mut size)?;

//     let file_slide = seg_linkedit.vmaddr - seg_text.vmaddr -
// seg_linkedit.fileoff;     let strings = library_header_address +
// symtab.stroff as u64 + file_slide;     let sym_addr = library_header_address
// + symtab.symoff as u64 + file_slide;

//     Ok(SymbolIter {
//         task,
//         nsyms: symtab.nsyms,
//         sym_addr,
//         strings,
//         shared_image_cache_slide,
//     })
// }
