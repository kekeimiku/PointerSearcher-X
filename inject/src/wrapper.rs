use std::{fs::File, io::Read, mem, os::unix::prelude::OsStrExt, path::Path};

use machx::{
    mach_types::thread_act_t,
    structs::arm_unified_thread_state_t,
    thread_status::{thread_state_t, ARM_THREAD_STATE64},
    vm_prot::{VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_statistics::VM_FLAGS_ANYWHERE,
    vm_types::mach_vm_address_t,
};

use super::{
    error::Error,
    ffi::{
        mach_vm_allocate, mach_vm_protect, mach_vm_write, task_for_pid, thread_create_running, ARM_THREAD_STATE64_COUNT,
    },
    utils::{find_library_addr, find_symbol_addr, gen_code},
};

pub fn inject<P: AsRef<Path>>(pid: i32, path: P) -> Result<(), Error> {
    let mut header = [0; 4];
    if !File::open(&path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_or(false, |_| [0xCF, 0xFA, 0xED, 0xFE].eq(&header))
    {
        return Err("invalid file".into());
    }
    let path = path.as_ref().as_os_str().as_bytes();
    unsafe { inj(path, pid) }
}

unsafe fn inj(path: &[u8], pid: i32) -> Result<(), Error> {
    let stack_size: u64 = 0x4000;
    let remote_task = task_for_pid(pid)?;

    let libdyld = find_library_addr(remote_task, "libdyld.dylib")?.ok_or("find libdyld.dylib failed.")?;

    let libsystem_pthread =
        find_library_addr(remote_task, "libsystem_pthread.dylib")?.ok_or("find libsystem_pthread.dylib failed.")?;

    let libsystem_kernel =
        find_library_addr(remote_task, "libsystem_kernel.dylib")?.ok_or("find libsystem_kernel.dylib failed.")?;

    let dlopen = find_symbol_addr(remote_task, libdyld, "_dlopen")?.ok_or("find dlopen failed.")?;

    let pthread_create_from_mach_thread =
        find_symbol_addr(remote_task, libsystem_pthread, "_pthread_create_from_mach_thread")?
            .ok_or("find _pthread_create_from_mach_thread failed.")?;

    let pthread_set_self = find_symbol_addr(remote_task, libsystem_pthread, "__pthread_set_self")?
        .ok_or("find __pthread_set_self failed.")?;

    let thread_suspend =
        find_symbol_addr(remote_task, libsystem_kernel, "_thread_suspend")?.ok_or("find _thread_suspend failed.")?;

    let mach_thread_self = find_symbol_addr(remote_task, libsystem_kernel, "_mach_thread_self")?
        .ok_or("find _mach_thread_self failed.")?;

    let mut remote_path: mach_vm_address_t = 0;
    mach_vm_allocate(remote_task, &mut remote_path, (path.len() + 1) as _, VM_FLAGS_ANYWHERE)?;

    mach_vm_write(remote_task, remote_path, path.as_ptr() as _, path.len() as _)?;

    mach_vm_protect(remote_task, remote_path, (path.len() + 1) as _, 0, VM_PROT_READ | VM_PROT_WRITE)?;

    let asm = gen_code(dlopen);
    let mut remote_code: mach_vm_address_t = 0;
    mach_vm_allocate(remote_task, &mut remote_code, asm.len() as _, VM_FLAGS_ANYWHERE)?;

    mach_vm_write(remote_task, remote_code, asm.as_ptr() as _, asm.len() as _)?;

    mach_vm_protect(remote_task, remote_code, asm.len() as _, 0, VM_PROT_READ | VM_PROT_EXECUTE)?;

    let mut remote_stack: mach_vm_address_t = 0;
    mach_vm_allocate(remote_task, &mut remote_stack, stack_size, VM_FLAGS_ANYWHERE)?;

    mach_vm_protect(remote_task, remote_stack, stack_size, 1, VM_PROT_READ | VM_PROT_WRITE)?;

    let parameters: [u64; 5] = [
        remote_code + (asm.len() as u64 - 24),
        pthread_set_self,
        pthread_create_from_mach_thread,
        thread_suspend,
        mach_thread_self,
    ];

    let mut remote_parameters: mach_vm_address_t = 0;
    mach_vm_allocate(remote_task, &mut remote_parameters, mem::size_of_val(&parameters) as u64, VM_FLAGS_ANYWHERE)?;

    mach_vm_write(remote_task, remote_parameters, parameters.as_ptr() as _, mem::size_of_val(&parameters) as _)?;

    let local_stack = remote_stack;
    remote_stack += stack_size / 2;

    let mut state = mem::zeroed::<arm_unified_thread_state_t>();

    state.uts.ts_64.__x[0] = remote_parameters;
    state.uts.ts_64.__x[1] = remote_path;
    state.uts.ts_64.__pc = remote_code;
    state.uts.ts_64.__sp = remote_stack;
    state.uts.ts_64.__lr = local_stack;

    let mut thread: thread_act_t = 0;
    thread_create_running(
        remote_task,
        ARM_THREAD_STATE64,
        &mut state.uts.ts_64 as *mut _ as thread_state_t,
        ARM_THREAD_STATE64_COUNT,
        &mut thread,
    )?;

    Ok(())
}
