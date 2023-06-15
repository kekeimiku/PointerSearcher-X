use std::{fs::File, io::Read, mem, os::unix::prelude::OsStrExt, path::Path, time::Duration};

use super::{
    bindgen::{
        arm_thread_state64_t, arm_unified_thread_state_t, mach_port_t, mach_vm_address_t, thread_act_t,
        thread_basic_info_data_t, thread_info_t, thread_state_t, ARM_THREAD_STATE64, THREAD_BASIC_INFO,
        VM_FLAGS_ANYWHERE,
    },
    error::Error,
    ffi::{
        mach_vm_allocate, mach_vm_protect, mach_vm_write, thread_create_running, thread_get_state, thread_info,
        thread_terminate, ARM_THREAD_STATE64_COUNT, THREAD_BASIC_INFO_COUNT, VM_PROT_EXECUTE, VM_PROT_READ,
        VM_PROT_WRITE,
    },
    utils,
};

pub fn find_library_address(task: mach_port_t, library: &str) -> Result<u64, Error> {
    unsafe { utils::find_library(task, library) }?.ok_or("library not found".into())
}

pub fn find_symbol_address(
    task: mach_port_t,
    library_header_address: mach_vm_address_t,
    symbol: &str,
) -> Result<u64, Error> {
    unsafe { utils::find_symbol(task, library_header_address, symbol) }?.ok_or("symbol not found".into())
}

pub fn task_for_pid(pid: i32) -> Result<mach_port_t, Error> {
    Ok(unsafe { super::ffi::task_for_pid(pid) }?)
}

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

    let libdyld = find_library_address(remote_task, "libdyld")?;

    let libsystem_pthread = find_library_address(remote_task, "libsystem_pthread")?;

    let libsystem_kernel = find_library_address(remote_task, "libsystem_kernel")?;

    let dlopen = find_symbol_address(remote_task, libdyld, "_dlopen")?;

    let pthread_create_from_mach_thread =
        find_symbol_address(remote_task, libsystem_pthread, "_pthread_create_from_mach_thread")?;

    let pthread_set_self = find_symbol_address(remote_task, libsystem_pthread, "__pthread_set_self")?;

    let thread_suspend = find_symbol_address(remote_task, libsystem_kernel, "_thread_suspend")?;

    let mach_thread_self = find_symbol_address(remote_task, libsystem_kernel, "_mach_thread_self")?;

    let mut remote_path: mach_vm_address_t = 0;
    mach_vm_allocate(remote_task, &mut remote_path, (path.len() + 1) as _, VM_FLAGS_ANYWHERE)?;

    mach_vm_write(remote_task, remote_path, path.as_ptr() as _, path.len() as _)?;

    mach_vm_protect(remote_task, remote_path, (path.len() + 1) as _, 0, VM_PROT_READ | VM_PROT_WRITE)?;

    let asm = utils::gen_asm(dlopen);
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

    state.ash.flavor = ARM_THREAD_STATE64;
    state.ash.count = ARM_THREAD_STATE64_COUNT;
    state.uts.ts_64.__x[0] = remote_parameters;
    state.uts.ts_64.__x[1] = remote_path;
    state.uts.ts_64.__pc = remote_code;
    state.uts.ts_64.__sp = remote_stack;
    state.uts.ts_64.__lr = local_stack;

    let mut thread: thread_act_t = 0;
    thread_create_running(
        remote_task,
        ARM_THREAD_STATE64 as _,
        &mut state.uts.ts_64 as *mut _ as thread_state_t,
        ARM_THREAD_STATE64_COUNT,
        &mut thread,
    )?;

    std::thread::sleep(Duration::from_millis(30));

    let mut basic_info = mem::zeroed::<thread_basic_info_data_t>();

    let mut info_count = THREAD_BASIC_INFO_COUNT;

    thread_info(thread, THREAD_BASIC_INFO, &mut basic_info as *mut _ as thread_info_t, &mut info_count)?;

    if basic_info.suspend_count > 0 {
        let mut state = mem::zeroed::<arm_thread_state64_t>();
        let mut count = ARM_THREAD_STATE64_COUNT;
        thread_get_state(thread, ARM_THREAD_STATE64 as _, &mut state as *mut _ as thread_state_t, &mut count)?;

        let result = state.__x[10];
        let thread_id = state.__x[11];

        if result == 0 && thread_id != 0 {
            std::thread::sleep(Duration::from_millis(30));
            thread_terminate(thread)?;
            return Ok(());
        }
    }

    Err(Error::Other("inject error".into()))
}
