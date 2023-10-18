use std::{ffi::CStr, mem, path::Path};

use machx::{
    dyld_images::{dyld_all_image_infos, dyld_image_info, mach_header_64, segment_command_64},
    kern_return::kern_return_t,
    loader::{load_command, symtab_command, LC_SEGMENT, LC_SEGMENT_64, LC_SYMTAB, SEG_LINKEDIT, SEG_TEXT},
    message::mach_msg_type_number_t,
    nlist::nlist_64,
    port::mach_port_t,
    task_info::{task_dyld_info_data_t, TASK_DYLD_INFO},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use super::ffi::{mach_vm_read_overwrite, task_info, TASK_DYLD_INFO_COUNT};

#[inline]
pub unsafe fn gen_code(dlopen: u64) -> [u8; 136] {
    // http://shell-storm.org/online/Online-Assembler-and-Disassembler/?opcodes=FD7BBDA9F50B00F9F44F02A9FD030091024C40A9085041A9151040F9BF0F00F9E30301AAE1031FAAA063009100013FD6EA0300AAA00F40F9EB0300AA60023FD6A0023FD680023FD6A00F40F9F44F42A9F50B40F9FD7BC3A8C0035FD61F2003D51F2003D51F2003D51F2003D51F2003D541008052E2DD97D2A2D5BBF20200D6F20200F4F240001FD6&arch=arm64&endianness=little&baddr=0x00000000&dis_with_addr=True&dis_with_raw=True&dis_with_ins=True#disassembly
    let mut code: [u8; 136] = [
        0xFD, 0x7B, 0xBD, 0xA9, 0xF5, 0x0B, 0x00, 0xF9, 0xF4, 0x4F, 0x02, 0xA9, 0xFD, 0x03, 0x00, 0x91, 0x02, 0x4C,
        0x40, 0xA9, 0x08, 0x50, 0x41, 0xA9, 0x15, 0x10, 0x40, 0xF9, 0xBF, 0x0F, 0x00, 0xF9, 0xE3, 0x03, 0x01, 0xAA,
        0xE1, 0x03, 0x1F, 0xAA, 0xA0, 0x63, 0x00, 0x91, 0x00, 0x01, 0x3F, 0xD6, 0xEA, 0x03, 0x00, 0xAA, 0xA0, 0x0F,
        0x40, 0xF9, 0xEB, 0x03, 0x00, 0xAA, 0x60, 0x02, 0x3F, 0xD6, 0xA0, 0x02, 0x3F, 0xD6, 0x80, 0x02, 0x3F, 0xD6,
        0xA0, 0x0F, 0x40, 0xF9, 0xF4, 0x4F, 0x42, 0xA9, 0xF5, 0x0B, 0x40, 0xF9, 0xFD, 0x7B, 0xC3, 0xA8, 0xC0, 0x03,
        0x5F, 0xD6, 0x1F, 0x20, 0x03, 0xD5, 0x1F, 0x20, 0x03, 0xD5, 0x1F, 0x20, 0x03, 0xD5, 0x1F, 0x20, 0x03, 0xD5,
        0x1F, 0x20, 0x03, 0xD5, 0x41, 0x00, 0x80, 0x52, 0xE2, 0xDD, 0x97, 0xD2, 0xA2, 0xD5, 0xBB, 0xF2, 0x02, 0x00,
        0xD6, 0xF2, 0x02, 0x00, 0xF4, 0xF2, 0x40, 0x00, 0x1F, 0xD6,
    ];

    #[inline]
    unsafe fn set_bits(reg: &mut [u8], value: u16) {
        let mut reg_u32 = u32::from_le_bytes(*(reg.as_ptr().cast()));
        for i in 0..=15 {
            let bit_to_set = ((value >> i) & 1) != 0;
            reg_u32 &= !(1 << (i + 5));
            reg_u32 |= (bit_to_set as u32) << (i + 5);
        }
        reg.as_mut_ptr()
            .copy_from_nonoverlapping(reg_u32.to_le_bytes().as_ptr(), 4);
    }

    let beef = (dlopen & 0x000000000000FFFF) as u16;
    let dead = ((dlopen & 0x00000000FFFF0000) >> 16) as u16;
    let b000 = ((dlopen & 0x0000FFFF00000000) >> 32) as u16;
    let a000 = ((dlopen & 0xFFFF000000000000) >> 48) as u16;

    set_bits(code.get_unchecked_mut(116..120), beef);
    set_bits(code.get_unchecked_mut(120..124), dead);
    set_bits(code.get_unchecked_mut(124..128), b000);
    set_bits(code.get_unchecked_mut(128..132), a000);

    code
}

#[inline]
pub unsafe fn find_library_addr(task: mach_port_t, library: &str) -> Result<Option<u64>, kern_return_t> {
    let mut dyld_info: task_dyld_info_data_t = mem::zeroed();
    let mut count: mach_msg_type_number_t = TASK_DYLD_INFO_COUNT as _;

    task_info(task, TASK_DYLD_INFO, &mut dyld_info as *mut task_dyld_info_data_t as _, &mut count)?;

    let mut image_infos = mem::zeroed::<dyld_all_image_infos>();
    let mut size = mem::size_of_val(&image_infos) as u64;

    mach_vm_read_overwrite(
        task,
        dyld_info.all_image_info_addr,
        size,
        &mut image_infos as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    let mut modules = vec![mem::zeroed::<dyld_image_info>(); image_infos.infoArrayCount as usize];
    size = (mem::size_of::<dyld_image_info>() * image_infos.infoArrayCount as usize) as mach_vm_size_t;

    mach_vm_read_overwrite(
        task,
        image_infos.infoArray as mach_vm_address_t,
        size,
        modules.as_mut_ptr() as mach_vm_address_t,
        &mut size,
    )?;

    let mut buf = [0; 512];

    for info in modules {
        mach_vm_read_overwrite(task, info.imageFilePath as _, buf.len() as _, buf.as_mut_ptr() as _, &mut size)?;
        let path = Path::new(std::str::from_utf8_unchecked(CStr::from_ptr(buf.as_ptr()).to_bytes()));
        if let Some(filename) = path.file_name() {
            if filename.eq(library) {
                return Ok(Some(info.imageLoadAddress as u64));
            }
        }
    }

    Ok(None)
}

#[inline]
pub unsafe fn find_symbol_addr(
    task: mach_port_t,
    library_header_address: mach_vm_address_t,
    symbol: &str,
) -> Result<Option<u64>, kern_return_t> {
    let mut header = mem::zeroed::<mach_header_64>();
    let mut size: mach_vm_size_t = mem::size_of_val(&header) as u64;

    mach_vm_read_overwrite(task, library_header_address, size, &mut header as *mut _ as mach_vm_address_t, &mut size)?;

    let mut info = mem::zeroed::<task_dyld_info_data_t>();
    let mut count: mach_msg_type_number_t = TASK_DYLD_INFO_COUNT as _;

    task_info(task, TASK_DYLD_INFO, &mut info as *mut task_dyld_info_data_t as _, &mut count)?;

    let mut infos = mem::zeroed::<dyld_all_image_infos>();
    let mut size = info.all_image_info_size;

    mach_vm_read_overwrite(
        task,
        info.all_image_info_addr,
        info.all_image_info_size,
        &mut infos as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    let shared_image_cache_slide = infos.sharedCacheSlide as u64;

    let mut command = mem::zeroed::<load_command>();
    let mut size = mem::size_of_val(&command) as u64;

    let mut seg_linkedit_addr: mach_vm_address_t = 0;
    let mut seg_text_addr: mach_vm_address_t = 0;
    let mut symtab_addr: mach_vm_address_t = 0;
    let mut load_command_address = library_header_address + mem::size_of::<mach_header_64>() as u64;

    for _ in 0..header.ncmds {
        mach_vm_read_overwrite(
            task,
            load_command_address,
            size,
            &mut command as *mut _ as mach_vm_address_t,
            &mut size,
        )?;

        if command.cmd == LC_SEGMENT | LC_SEGMENT_64 {
            let mut name = [0; 512];
            let mut size = mem::size_of_val(&name) as u64;

            let segname_offset = mem::offset_of!(segment_command_64, segname);

            mach_vm_read_overwrite(
                task,
                load_command_address + segname_offset as u64,
                size,
                name.as_mut_ptr() as mach_vm_address_t,
                &mut size,
            )?;

            let seg_name = CStr::from_ptr(name.as_ptr()).to_bytes_with_nul();

            if seg_name == SEG_TEXT {
                seg_text_addr = load_command_address;
            } else if seg_name == SEG_LINKEDIT {
                seg_linkedit_addr = load_command_address;
            }
        }

        if command.cmd == LC_SYMTAB {
            symtab_addr = load_command_address;
        }

        load_command_address += command.cmdsize as u64;
    }

    let mut seg_linkedit = mem::zeroed::<segment_command_64>();
    let mut seg_text = mem::zeroed::<segment_command_64>();
    let mut symtab = mem::zeroed::<symtab_command>();

    let mut size = mem::size_of_val(&seg_linkedit) as u64;

    mach_vm_read_overwrite(task, seg_linkedit_addr, size, &mut seg_linkedit as *mut _ as mach_vm_address_t, &mut size)?;

    let mut size = mem::size_of_val(&seg_text) as u64;

    mach_vm_read_overwrite(task, seg_text_addr, size, &mut seg_text as *mut _ as mach_vm_address_t, &mut size)?;

    let mut size = mem::size_of_val(&symtab) as u64;

    mach_vm_read_overwrite(task, symtab_addr, size, &mut symtab as *mut _ as mach_vm_address_t, &mut size)?;

    let file_slide = seg_linkedit.vmaddr - seg_text.vmaddr - seg_linkedit.fileoff;
    let strings = library_header_address + symtab.stroff as u64 + file_slide;
    let mut sym_addr = library_header_address + symtab.symoff as u64 + file_slide;

    let mut name = [0; 512];

    let mut sym = mem::zeroed::<nlist_64>();
    let mut sym_size = mem::size_of_val(&sym) as u64;

    for _ in 0..symtab.nsyms {
        mach_vm_read_overwrite(task, sym_addr, sym_size, &mut sym as *mut _ as mach_vm_address_t, &mut sym_size)?;

        let mut size = mem::size_of_val(&name) as u64;
        let symname_offset = strings + sym.n_un.n_strx as u64;

        mach_vm_read_overwrite(task, symname_offset, size, name.as_mut_ptr() as mach_vm_address_t, &mut size)?;

        let sym_name = CStr::from_ptr(name.as_ptr());

        if sym_name.to_bytes() == symbol.as_bytes() {
            return Ok(Some(sym.n_value + shared_image_cache_slide));
        }

        sym_addr += sym_size;
    }

    Ok(None)
}
