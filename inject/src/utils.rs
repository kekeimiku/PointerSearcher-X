use std::{ffi::CStr, mem, slice};

use super::{
    bindgen::{
        dyld_all_image_infos, dyld_image_info, kern_return_t, load_command, mach_header_64,
        mach_msg_type_number_t, mach_port_t, mach_vm_address_t, mach_vm_size_t, nlist_64,
        segment_command_64, symtab_command, task_dyld_info_data_t, LC_SEGMENT, LC_SEGMENT_64,
        LC_SYMTAB, MH_MAGIC_64, SEG_LINKEDIT, SEG_TEXT, TASK_DYLD_INFO,
    },
    ffi::{mach_vm_read_overwrite, task_info, PIDPATHINFO_MAXSIZE, TASK_DYLD_INFO_COUNT},
};

pub const GLOBALASM: &[u8; 136] = include_bytes!("aarch64");

pub fn gen_asm(dlopen: u64) -> [u8; 136] {
    let mut asm: [u8; 136] = *GLOBALASM;
    let asm_size = asm.len();

    let copy_bits = |reg: &mut u32, value: u16| {
        for (bit, value_bit) in (5..=20).rev().zip((0..=15).rev()) {
            let bit_to_set = ((value >> value_bit) & 1) != 0;
            *reg ^= ((bit_to_set as u32).wrapping_neg() ^ *reg) & (1u32 << bit);
        }
    };

    let decode_instruction = |instructions: &[u8]| -> u32 {
        (instructions[3] as u32) << 24
            | (instructions[2] as u32) << 16
            | (instructions[1] as u32) << 8
            | (instructions[0] as u32)
    };

    let encode_instruction = |instruction: u32, instructions: &mut [u8; 4]| {
        instructions[3] = ((instruction & 0xFF000000) >> 24) as u8;
        instructions[2] = ((instruction & 0x00FF0000) >> 16) as u8;
        instructions[1] = ((instruction & 0x0000FF00) >> 8) as u8;
        instructions[0] = (instruction & 0x000000FF) as u8;
    };

    let write_instruction_address = |address_intermediate: u32, asm: &mut [u8], offset: isize| {
        let mut instructions = [0u8; 4];
        instructions.copy_from_slice(
            &asm[(asm_size as isize + offset) as usize..(asm_size as isize + offset + 4) as usize],
        );

        let mut instruction = decode_instruction(&instructions);
        copy_bits(&mut instruction, (address_intermediate & 0xFFFF) as u16);

        encode_instruction(instruction, &mut instructions);

        asm[(asm_size as isize + offset) as usize..(asm_size as isize + offset + 4) as usize]
            .copy_from_slice(&instructions);
    };

    let beef = (dlopen & 0x000000000000FFFF) as u32;
    let dead = ((dlopen & 0x00000000FFFF0000) >> 16) as u32;
    let b000 = ((dlopen & 0x0000FFFF00000000) >> 32) as u32;
    let a000 = ((dlopen & 0xFFFF000000000000) >> 48) as u32;

    write_instruction_address(a000, &mut asm, -8);
    write_instruction_address(b000, &mut asm, -12);
    write_instruction_address(dead, &mut asm, -16);
    write_instruction_address(beef, &mut asm, -20);

    asm
}

pub unsafe fn find_library(task: mach_port_t, library: &str) -> Result<Option<u64>, kern_return_t> {
    let mut info: task_dyld_info_data_t = mem::zeroed();
    let mut count: mach_msg_type_number_t = TASK_DYLD_INFO_COUNT as _;

    task_info(
        task,
        TASK_DYLD_INFO,
        &mut info as *mut task_dyld_info_data_t as _,
        &mut count,
    )?;

    let mut infos = mem::zeroed::<dyld_all_image_infos>();
    let mut size = info.all_image_info_size;

    mach_vm_read_overwrite(
        task,
        info.all_image_info_addr,
        info.all_image_info_size,
        &mut infos as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    size = mem::size_of::<dyld_all_image_infos>() as u64 * infos.infoArrayCount as u64;

    let image_infos = {
        let mut vec = Vec::<dyld_image_info>::with_capacity(size as _);
        let ptr = vec.as_mut_ptr();
        mem::forget(vec);
        slice::from_raw_parts_mut(ptr, size as _)
    };

    mach_vm_read_overwrite(
        task,
        infos.infoArray as *const _ as mach_vm_address_t,
        size,
        image_infos.as_mut_ptr() as mach_vm_address_t,
        &mut size,
    )?;

    let mut buf = [0; PIDPATHINFO_MAXSIZE];

    for info in image_infos.iter_mut().take(infos.infoArrayCount as usize) {
        mach_vm_read_overwrite(
            task,
            info.imageFilePath as _,
            buf.len() as _,
            buf.as_mut_ptr() as _,
            &mut size,
        )?;

        let path = CStr::from_ptr(buf.as_ptr()).to_bytes_with_nul();
        let library = library.as_bytes();
        if path.windows(library.len()).any(|w| w == library) {
            return Ok(Some(info.imageLoadAddress as u64));
        }
    }

    Ok(None)
}

pub unsafe fn find_symbol(
    task: mach_port_t,
    library_header_address: mach_vm_address_t,
    symbol: &str,
) -> Result<Option<u64>, kern_return_t> {
    let mut header = mem::zeroed::<mach_header_64>();
    let mut size: mach_vm_size_t = mem::size_of_val(&header) as u64;

    mach_vm_read_overwrite(
        task,
        library_header_address,
        size,
        &mut header as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    if header.magic != MH_MAGIC_64 {
        return Ok(None);
    }

    let mut shared_image_cache_slide = 0;
    if (header.flags & 0x80000000) == 0x80000000 {
        let mut info = mem::zeroed::<task_dyld_info_data_t>();
        let mut count: mach_msg_type_number_t = TASK_DYLD_INFO_COUNT as _;

        task_info(
            task,
            TASK_DYLD_INFO,
            &mut info as *mut task_dyld_info_data_t as _,
            &mut count,
        )?;

        let mut infos = mem::zeroed::<dyld_all_image_infos>();
        let mut size = info.all_image_info_size;

        mach_vm_read_overwrite(
            task,
            info.all_image_info_addr,
            info.all_image_info_size,
            &mut infos as *mut _ as mach_vm_address_t,
            &mut size,
        )?;
        shared_image_cache_slide = infos.sharedCacheSlide as u64
    }

    let mut command = mem::zeroed::<load_command>();
    size = mem::size_of_val(&command) as u64;

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
            let mut name: [std::os::raw::c_char; 512] = [0; 512];
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

    if seg_text_addr == 0 || seg_linkedit_addr == 0 || symtab_addr == 0 {
        return Ok(None);
    }

    let mut seg_linkedit = mem::zeroed::<segment_command_64>();
    let mut seg_text = mem::zeroed::<segment_command_64>();
    let mut symtab = mem::zeroed::<symtab_command>();

    let mut size = mem::size_of_val(&seg_linkedit) as u64;

    mach_vm_read_overwrite(
        task,
        seg_linkedit_addr,
        size,
        &mut seg_linkedit as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    size = mem::size_of_val(&seg_text) as u64;

    mach_vm_read_overwrite(
        task,
        seg_text_addr,
        size,
        &mut seg_text as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    size = mem::size_of_val(&symtab) as u64;

    mach_vm_read_overwrite(
        task,
        symtab_addr,
        size,
        &mut symtab as *mut _ as mach_vm_address_t,
        &mut size,
    )?;

    let file_slide = seg_linkedit.vmaddr - seg_text.vmaddr - seg_linkedit.fileoff;
    let strings = library_header_address + symtab.stroff as u64 + file_slide;
    let mut sym_addr = library_header_address + symtab.symoff as u64 + file_slide;

    let mut name = [0; PIDPATHINFO_MAXSIZE];

    for _ in 0..symtab.nsyms {
        let mut sym = mem::zeroed::<nlist_64>();
        size = mem::size_of_val(&sym) as u64;

        mach_vm_read_overwrite(
            task,
            sym_addr,
            size,
            &mut sym as *mut _ as mach_vm_address_t,
            &mut size,
        )?;

        if sym.n_value != 0 {
            let mut size = mem::size_of_val(&name) as u64;

            let symname_offset = strings + sym.n_un.n_strx as u64;

            mach_vm_read_overwrite(
                task,
                symname_offset,
                size,
                name.as_mut_ptr() as mach_vm_address_t,
                &mut size,
            )?;

            let sym_name = CStr::from_ptr(name.as_ptr());

            if sym_name.to_bytes() == symbol.as_bytes() {
                if sym.n_value < 0x1000 {
                    return Ok(Some(sym.n_value + library_header_address));
                }

                if shared_image_cache_slide != 0 {
                    return Ok(Some(sym.n_value + shared_image_cache_slide));
                }

                return Ok(Some(sym.n_value + library_header_address));
            }
        }

        sym_addr += size;
    }

    Ok(None)
}
