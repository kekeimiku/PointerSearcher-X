// TODO

#![allow(non_camel_case_types, non_snake_case)]

pub type WIN32_ERROR = u32;
pub type HANDLE = isize;
pub type BOOL = i32;
pub type NTSTATUS = i32;
pub type LPTHREAD_START_ROUTINE =
    ::core::option::Option<unsafe extern "system" fn(lpthreadparameter: *mut ::core::ffi::c_void) -> u32>;
pub type PSTR = *mut u8;
pub type PWSTR = *mut u16;
pub type RIP_INFO_TYPE = u32;
pub type DEBUG_EVENT_CODE = u32;
pub type CREATE_TOOLHELP_SNAPSHOT_FLAGS = u32;
pub const INFINITE: u32 = 4294967295u32;
pub const DBG_CONTINUE: NTSTATUS = 65538i32;
pub const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = -2147418111i32;
pub const INVALID_HANDLE_VALUE: HANDLE = -1i32 as _;
pub const TH32CS_SNAPTHREAD: CREATE_TOOLHELP_SNAPSHOT_FLAGS = 4u32;
pub type THREAD_ACCESS_RIGHTS = u32;
pub const THREAD_SUSPEND_RESUME: THREAD_ACCESS_RIGHTS = 2u32;
pub const THREAD_GET_CONTEXT: THREAD_ACCESS_RIGHTS = 8u32;
pub const THREAD_SET_CONTEXT: THREAD_ACCESS_RIGHTS = 16u32;
pub const THREAD_QUERY_INFORMATION: THREAD_ACCESS_RIGHTS = 64u32;
pub const EXCEPTION_DEBUG_EVENT: DEBUG_EVENT_CODE = 1u32;
pub const EXCEPTION_SINGLE_STEP: NTSTATUS = -2147483644i32;
pub type PAGE_PROTECTION_FLAGS = u32;
pub type VIRTUAL_ALLOCATION_TYPE = u32;
pub type PAGE_TYPE = u32;
pub const PAGE_EXECUTE_READWRITE: PAGE_PROTECTION_FLAGS = 64u32;
pub const PAGE_EXECUTE_WRITECOPY: PAGE_PROTECTION_FLAGS = 128u32;
pub const PAGE_READWRITE: PAGE_PROTECTION_FLAGS = 4u32;
pub const PAGE_WRITECOPY: PAGE_PROTECTION_FLAGS = 8u32;
pub type PROCESS_ACCESS_RIGHTS = u32;
pub const CONTEXT_ALL: u32 = 1_048_607;
pub const PROCESS_ALL_ACCESS: PROCESS_ACCESS_RIGHTS = 2097151u32;
pub type HINSTANCE = isize;
pub const TH32CS_SNAPMODULE: CREATE_TOOLHELP_SNAPSHOT_FLAGS = 8u32;
pub const TH32CS_SNAPMODULE32: CREATE_TOOLHELP_SNAPSHOT_FLAGS = 16u32;
pub const PAGE_EXECUTE_READ: PAGE_PROTECTION_FLAGS = 32u32;
pub const PAGE_READONLY: PAGE_PROTECTION_FLAGS = 2u32;

#[repr(C)]
pub struct EXCEPTION_RECORD {
    pub ExceptionCode: NTSTATUS,
    pub ExceptionFlags: u32,
    pub ExceptionRecord: *mut EXCEPTION_RECORD,
    pub ExceptionAddress: *mut ::core::ffi::c_void,
    pub NumberParameters: u32,
    pub ExceptionInformation: [usize; 15],
}
impl Copy for EXCEPTION_RECORD {}
impl Clone for EXCEPTION_RECORD {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct EXCEPTION_DEBUG_INFO {
    pub ExceptionRecord: EXCEPTION_RECORD,
    pub dwFirstChance: u32,
}
impl Copy for EXCEPTION_DEBUG_INFO {}
impl Clone for EXCEPTION_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct CREATE_THREAD_DEBUG_INFO {
    pub hThread: HANDLE,
    pub lpThreadLocalBase: *mut ::core::ffi::c_void,
    pub lpStartAddress: LPTHREAD_START_ROUTINE,
}
impl Copy for CREATE_THREAD_DEBUG_INFO {}
impl Clone for CREATE_THREAD_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct CREATE_PROCESS_DEBUG_INFO {
    pub hFile: HANDLE,
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub lpBaseOfImage: *mut ::core::ffi::c_void,
    pub dwDebugInfoFileOffset: u32,
    pub nDebugInfoSize: u32,
    pub lpThreadLocalBase: *mut ::core::ffi::c_void,
    pub lpStartAddress: LPTHREAD_START_ROUTINE,
    pub lpImageName: *mut ::core::ffi::c_void,
    pub fUnicode: u16,
}
impl Copy for CREATE_PROCESS_DEBUG_INFO {}
impl Clone for CREATE_PROCESS_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct EXIT_THREAD_DEBUG_INFO {
    pub dwExitCode: u32,
}
impl Copy for EXIT_THREAD_DEBUG_INFO {}
impl Clone for EXIT_THREAD_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct EXIT_PROCESS_DEBUG_INFO {
    pub dwExitCode: u32,
}
impl Copy for EXIT_PROCESS_DEBUG_INFO {}
impl Clone for EXIT_PROCESS_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct LOAD_DLL_DEBUG_INFO {
    pub hFile: HANDLE,
    pub lpBaseOfDll: *mut ::core::ffi::c_void,
    pub dwDebugInfoFileOffset: u32,
    pub nDebugInfoSize: u32,
    pub lpImageName: *mut ::core::ffi::c_void,
    pub fUnicode: u16,
}
impl Copy for LOAD_DLL_DEBUG_INFO {}
impl Clone for LOAD_DLL_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct UNLOAD_DLL_DEBUG_INFO {
    pub lpBaseOfDll: *mut ::core::ffi::c_void,
}
impl Copy for UNLOAD_DLL_DEBUG_INFO {}
impl Clone for UNLOAD_DLL_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct OUTPUT_DEBUG_STRING_INFO {
    pub lpDebugStringData: PSTR,
    pub fUnicode: u16,
    pub nDebugStringLength: u16,
}
impl Copy for OUTPUT_DEBUG_STRING_INFO {}
impl Clone for OUTPUT_DEBUG_STRING_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct RIP_INFO {
    pub dwError: u32,
    pub dwType: RIP_INFO_TYPE,
}
impl Copy for RIP_INFO {}
impl Clone for RIP_INFO {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub union DEBUG_EVENT_0 {
    pub Exception: EXCEPTION_DEBUG_INFO,
    pub CreateThread: CREATE_THREAD_DEBUG_INFO,
    pub CreateProcessInfo: CREATE_PROCESS_DEBUG_INFO,
    pub ExitThread: EXIT_THREAD_DEBUG_INFO,
    pub ExitProcess: EXIT_PROCESS_DEBUG_INFO,
    pub LoadDll: LOAD_DLL_DEBUG_INFO,
    pub UnloadDll: UNLOAD_DLL_DEBUG_INFO,
    pub DebugString: OUTPUT_DEBUG_STRING_INFO,
    pub RipInfo: RIP_INFO,
}
impl Copy for DEBUG_EVENT_0 {}
impl Clone for DEBUG_EVENT_0 {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct DEBUG_EVENT {
    pub dwDebugEventCode: DEBUG_EVENT_CODE,
    pub dwProcessId: u32,
    pub dwThreadId: u32,
    pub u: DEBUG_EVENT_0,
}
impl Copy for DEBUG_EVENT {}
impl Clone for DEBUG_EVENT {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct THREADENTRY32 {
    pub dwSize: u32,
    pub cntUsage: u32,
    pub th32ThreadID: u32,
    pub th32OwnerProcessID: u32,
    pub tpBasePri: i32,
    pub tpDeltaPri: i32,
    pub dwFlags: u32,
}

#[repr(C)]
pub struct M128A {
    pub Low: u64,
    pub High: i64,
}
impl Copy for M128A {}
impl Clone for M128A {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct XSAVE_FORMAT {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [M128A; 8],
    pub XmmRegisters: [M128A; 16],
    pub Reserved4: [u8; 96],
}
impl Copy for XSAVE_FORMAT {}
impl Clone for XSAVE_FORMAT {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub struct CONTEXT_0_0 {
    pub Header: [M128A; 2],
    pub Legacy: [M128A; 8],
    pub Xmm0: M128A,
    pub Xmm1: M128A,
    pub Xmm2: M128A,
    pub Xmm3: M128A,
    pub Xmm4: M128A,
    pub Xmm5: M128A,
    pub Xmm6: M128A,
    pub Xmm7: M128A,
    pub Xmm8: M128A,
    pub Xmm9: M128A,
    pub Xmm10: M128A,
    pub Xmm11: M128A,
    pub Xmm12: M128A,
    pub Xmm13: M128A,
    pub Xmm14: M128A,
    pub Xmm15: M128A,
}
impl Copy for CONTEXT_0_0 {}
impl Clone for CONTEXT_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub union CONTEXT_0 {
    pub FltSave: XSAVE_FORMAT,
    pub Anonymous: CONTEXT_0_0,
}

#[repr(C, align(16))]
#[cfg(target_arch = "x86_64")]
pub struct CONTEXT {
    pub P1Home: u64,
    pub P2Home: u64,
    pub P3Home: u64,
    pub P4Home: u64,
    pub P5Home: u64,
    pub P6Home: u64,
    pub ContextFlags: u32,
    pub MxCsr: u32,
    pub SegCs: u16,
    pub SegDs: u16,
    pub SegEs: u16,
    pub SegFs: u16,
    pub SegGs: u16,
    pub SegSs: u16,
    pub EFlags: u32,
    pub Dr0: u64,
    pub Dr1: u64,
    pub Dr2: u64,
    pub Dr3: u64,
    pub Dr6: u64,
    pub Dr7: u64,
    pub Rax: u64,
    pub Rcx: u64,
    pub Rdx: u64,
    pub Rbx: u64,
    pub Rsp: u64,
    pub Rbp: u64,
    pub Rsi: u64,
    pub Rdi: u64,
    pub R8: u64,
    pub R9: u64,
    pub R10: u64,
    pub R11: u64,
    pub R12: u64,
    pub R13: u64,
    pub R14: u64,
    pub R15: u64,
    pub Rip: u64,
    pub Anonymous: CONTEXT_0,
    pub VectorRegister: [M128A; 26],
    pub VectorControl: u64,
    pub DebugControl: u64,
    pub LastBranchToRip: u64,
    pub LastBranchFromRip: u64,
    pub LastExceptionToRip: u64,
    pub LastExceptionFromRip: u64,
}

#[repr(C)]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct MEMORY_BASIC_INFORMATION {
    pub BaseAddress: *mut ::core::ffi::c_void,
    pub AllocationBase: *mut ::core::ffi::c_void,
    pub AllocationProtect: PAGE_PROTECTION_FLAGS,
    pub PartitionId: u16,
    pub RegionSize: usize,
    pub State: VIRTUAL_ALLOCATION_TYPE,
    pub Protect: PAGE_PROTECTION_FLAGS,
    pub Type: PAGE_TYPE,
}

#[repr(C)]
pub struct MODULEENTRY32W {
    pub dwSize: u32,
    pub th32ModuleID: u32,
    pub th32ProcessID: u32,
    pub GlblcntUsage: u32,
    pub ProccntUsage: u32,
    pub modBaseAddr: *mut u8,
    pub modBaseSize: u32,
    pub hModule: HINSTANCE,
    pub szModule: [u16; 256],
    pub szExePath: [u16; 260],
}

#[cfg_attr(windows, link(name = "kernel32"))]
extern "system" {
    pub fn GetLastError() -> WIN32_ERROR;
    pub fn CloseHandle(hobject: HANDLE) -> BOOL;
    pub fn DebugActiveProcess(dwprocessid: u32) -> BOOL;
    pub fn DebugSetProcessKillOnExit(killonexit: BOOL) -> BOOL;
    pub fn WaitForDebugEvent(lpdebugevent: *mut DEBUG_EVENT, dwmilliseconds: u32) -> BOOL;
    pub fn ContinueDebugEvent(dwprocessid: u32, dwthreadid: u32, dwcontinuestatus: u32) -> BOOL;
    pub fn FlushInstructionCache(
        hprocess: HANDLE,
        lpbaseaddress: *const ::core::ffi::c_void,
        dwsize: usize,
    ) -> BOOL;
    pub fn CreateToolhelp32Snapshot(dwflags: CREATE_TOOLHELP_SNAPSHOT_FLAGS, th32processid: u32) -> HANDLE;
    pub fn Thread32First(hsnapshot: HANDLE, lpte: *mut THREADENTRY32) -> BOOL;
    pub fn Thread32Next(hsnapshot: HANDLE, lpte: *mut THREADENTRY32) -> BOOL;
    pub fn ResumeThread(hthread: HANDLE) -> u32;
    pub fn SetThreadContext(hthread: HANDLE, lpcontext: *const CONTEXT) -> BOOL;
    pub fn SuspendThread(hthread: HANDLE) -> u32;
    pub fn GetThreadContext(hthread: HANDLE, lpcontext: *mut CONTEXT) -> BOOL;
    pub fn OpenThread(dwdesiredaccess: THREAD_ACCESS_RIGHTS, binherithandle: BOOL, dwthreadid: u32)
        -> HANDLE;
    pub fn OpenProcess(
        dwdesiredaccess: PROCESS_ACCESS_RIGHTS,
        binherithandle: BOOL,
        dwprocessid: u32,
    ) -> HANDLE;
    pub fn ReadProcessMemory(
        hprocess: HANDLE,
        lpbaseaddress: *const ::core::ffi::c_void,
        lpbuffer: *mut ::core::ffi::c_void,
        nsize: usize,
        lpnumberofbytesread: *mut usize,
    ) -> BOOL;
    pub fn WriteProcessMemory(
        hprocess: HANDLE,
        lpbaseaddress: *const ::core::ffi::c_void,
        lpbuffer: *const ::core::ffi::c_void,
        nsize: usize,
        lpnumberofbyteswritten: *mut usize,
    ) -> BOOL;
    pub fn VirtualQueryEx(
        hprocess: HANDLE,
        lpaddress: *const ::core::ffi::c_void,
        lpbuffer: *mut MEMORY_BASIC_INFORMATION,
        dwlength: usize,
    ) -> usize;
    pub fn K32GetMappedFileNameW(
        hprocess: HANDLE,
        lpv: *const ::core::ffi::c_void,
        lpfilename: PWSTR,
        nsize: u32,
    ) -> u32;
    pub fn Module32FirstW(hsnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    pub fn Module32NextW(hsnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    pub fn K32GetModuleBaseNameW(hprocess: HANDLE, hmodule: HINSTANCE, lpbasename: PWSTR, nsize: u32) -> u32;
    pub fn K32EnumProcesses(lpidprocess: *mut u32, cb: u32, lpcbneeded: *mut u32) -> BOOL;
}

// 方便调用的ffi封装
pub type Result<T> = ::core::result::Result<T, WIN32_ERROR>;

pub fn debug_set_process_kill_on_exit(bool: i32) -> Result<()> {
    if unsafe { DebugSetProcessKillOnExit(bool) } == 0 {
        return Err(unsafe { GetLastError() });
    };
    Ok(())
}

pub fn debug_active_process(pid: u32) -> Result<()> {
    if unsafe { DebugActiveProcess(pid) } == 0 {
        return Err(unsafe { GetLastError() });
    };
    Ok(())
}

#[inline(always)]
pub fn wait_for_debug_event(timeout: u32) -> Result<DEBUG_EVENT> {
    let mut result = core::mem::MaybeUninit::uninit();

    if unsafe { WaitForDebugEvent(result.as_mut_ptr(), timeout) } == 0 {
        return Err(unsafe { GetLastError() });
    }
    Ok(unsafe { result.assume_init() })
}

#[inline(always)]
pub fn continue_debug_event(event: DEBUG_EVENT, handled: bool) -> Result<()> {
    if unsafe {
        ContinueDebugEvent(
            event.dwProcessId,
            event.dwThreadId,
            if handled {
                DBG_CONTINUE.try_into().unwrap()
            } else {
                DBG_EXCEPTION_NOT_HANDLED.try_into().unwrap()
            },
        )
    } == 0
    {
        return Err(unsafe { GetLastError() });
    }

    Ok(())
}

#[inline(always)]
pub fn flush_instruction_cache(handle: HANDLE) -> Result<()> {
    let ret = unsafe { FlushInstructionCache(handle, core::ptr::null(), 0) };
    if ret == 0 {
        return Err(unsafe { GetLastError() });
    }
    Ok(())
}

#[inline(always)]
pub fn create_toolhelp32_snapshot(dwflags: u32, th32processid: u32) -> Result<HANDLE> {
    let handle = unsafe { CreateToolhelp32Snapshot(dwflags, th32processid) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(unsafe { GetLastError() });
    }
    Ok(handle)
}

#[inline(always)]
pub fn thread32_first(handle: HANDLE, lpte: &mut THREADENTRY32) -> Result<()> {
    if unsafe { Thread32First(handle, lpte) } == 0 {
        return Err(unsafe { GetLastError() });
    }
    Ok(())
}

#[inline(always)]
pub fn thread32_next(handle: HANDLE, lpte: &mut THREADENTRY32) -> Result<()> {
    if unsafe { Thread32Next(handle, lpte) } == 0 {
        return Err(unsafe { GetLastError() });
    }
    Ok(())
}

#[inline(always)]
pub fn get_thread_context(handle: HANDLE) -> Result<CONTEXT> {
    let context = core::mem::MaybeUninit::<CONTEXT>::zeroed();
    let mut context = unsafe { context.assume_init() };
    context.ContextFlags = CONTEXT_ALL;
    if unsafe { GetThreadContext(handle, &mut context) } == 0 {
        return Err(unsafe { GetLastError() });
    }
    Ok(context)
}

#[inline(always)]
pub fn set_thread_context(handle: HANDLE, context: &CONTEXT) -> Result<()> {
    let ret = unsafe { SetThreadContext(handle, context) };
    if ret == 0 {
        return Err(unsafe { GetLastError() });
    };
    Ok(())
}

#[inline(always)]
pub fn suspend_thread(handle: HANDLE) -> Result<()> {
    let ret = unsafe { SuspendThread(handle) };
    if ret == -1i32 as u32 {
        return Err(unsafe { GetLastError() });
    }
    Ok(())
}

#[inline(always)]
pub fn resume_thread(handle: HANDLE) -> Result<()> {
    let ret = unsafe { ResumeThread(handle) };
    if ret == -1i32 as u32 {
        return Err(unsafe { GetLastError() });
    }
    Ok(())
}

#[inline(always)]
pub fn k32_get_module_base_name_w() {}

#[inline(always)]
pub fn k32_enum_processes() -> Vec<u32> {
    let mut pids = Vec::<u32>::with_capacity(1024);
    let mut size = 0;
    let code = unsafe { K32EnumProcesses(pids.as_mut_ptr(), pids.capacity() as u32, &mut size) };
    if code == 0 {
        println!("err: {}", unsafe { GetLastError() });
    }
    unsafe { pids.set_len(size as _) }

    pids
}

// TODO

use std::{ffi::OsString, mem, os::windows::prelude::OsStringExt};

use crate::{
    error::Result,
    ffi::windows::{
        create_toolhelp32_snapshot, thread32_first, thread32_next, K32GetMappedFileNameW, VirtualQueryEx,
        HANDLE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_READONLY, PAGE_READWRITE,
        TH32CS_SNAPTHREAD, THREADENTRY32,
    },
};

#[derive(Debug)]
pub struct Process {
    pub pid: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Maps {
    start: usize,
    end: usize,
    size: usize,
    pub flags: u32,
    pathname: String,
}

impl Maps {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn pathname(&self) -> &str {
        &self.pathname
    }
    pub fn is_exec(&self) -> bool {
        (self.flags & PAGE_EXECUTE_READ) != 0
    }
    pub fn is_write(&self) -> bool {
        (self.flags & PAGE_READWRITE) != 0
    }
    pub fn is_read(&self) -> bool {
        (self.flags & PAGE_READONLY) != 0
    }
}

pub struct MapsIter {
    handle: HANDLE,
    base: usize,
}

impl MapsIter {
    pub const fn new(handle: HANDLE) -> Self {
        Self { handle, base: 0 }
    }
}

pub const MAX_PATH: u32 = 260u32;

impl Iterator for MapsIter {
    type Item = Maps;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut basic = mem::MaybeUninit::uninit();

        let mut name = vec![0u16; MAX_PATH as usize];

        // 暂时先这样罢
        if unsafe {
            K32GetMappedFileNameW(self.handle, self.base as _, name.as_mut_ptr(), name.len() as _);
            VirtualQueryEx(self.handle, self.base as *const _, basic.as_mut_ptr(), mem::size_of::<Maps>())
        } != mem::size_of::<MEMORY_BASIC_INFORMATION>()
        {
            return None;
        }

        let info = unsafe { basic.assume_init() };
        self.base = info.BaseAddress as usize + info.RegionSize;

        Some(Maps {
            start: info.BaseAddress as _,
            end: info.BaseAddress as usize + info.RegionSize,
            size: info.RegionSize,
            flags: info.Protect,
            pathname: wstr_to_string(&name).to_string_lossy().to_string(),
        })
    }
}

fn wstr_to_string(full: &[u16]) -> OsString {
    let len = full.iter().position(|&x| x == 0).unwrap_or(full.len());
    OsString::from_wide(&full[..len])
}

#[inline(always)]
pub fn get_thread_list_by_pid(pid: u32) -> Result<Vec<u32>> {
    const ENTRY_SIZE: u32 = mem::size_of::<THREADENTRY32>() as u32;
    const NEEDED_ENTRY_SIZE: u32 = 4 * mem::size_of::<u32>() as u32;
    let handle = create_toolhelp32_snapshot(TH32CS_SNAPTHREAD, 0).unwrap();
    let mut result = Vec::new();
    let mut entry = THREADENTRY32 {
        dwSize: ENTRY_SIZE,
        cntUsage: 0,
        th32ThreadID: 0,
        th32OwnerProcessID: 0,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0,
    };

    if thread32_first(handle, &mut entry).is_ok() {
        loop {
            if entry.dwSize >= NEEDED_ENTRY_SIZE && entry.th32OwnerProcessID == pid {
                result.push(entry.th32ThreadID);
            }
            entry.dwSize = ENTRY_SIZE;
            if thread32_next(handle, &mut entry).is_err() {
                break;
            }
        }
    }

    Ok(result)
}

pub struct Mem(pub HANDLE);

impl MemExt for Mem {
    fn read(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size];
        let code = unsafe { ReadProcessMemory(self.0, addr as _, buf.as_mut_ptr() as _, size, null_mut()) };
        if code == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::GetLastError(error));
        }

        Ok(buf)
    }

    fn write(&self, addr: usize, payload: &[u8]) -> Result<usize> {
        let code = unsafe {
            WriteProcessMemory(
                self.0,
                addr as *mut _,
                payload.as_ptr() as *const _,
                payload.len(),
                null_mut(),
            )
        };

        if code == 0 {
            let error = unsafe { GetLastError() };
            return Err(Error::GetLastError(error));
        }

        Ok(payload.len())
    }
}

use crate::{
    ext::{DebugExt, ThreadExt},
    ffi::windows::{
        continue_debug_event, debug_active_process, debug_set_process_kill_on_exit, flush_instruction_cache,
        get_thread_context, resume_thread, set_thread_context, suspend_thread, wait_for_debug_event, CONTEXT,
        DEBUG_EVENT, EXCEPTION_DEBUG_EVENT, EXCEPTION_SINGLE_STEP, HANDLE, INFINITE,
    },
};

use super::ext::{Cond, Size};

use super::error::Result;

pub struct Dbg {
    pid: u32,
    handle: HANDLE,
}

impl Dbg {
    pub fn new(pid: u32, handle: HANDLE) -> Self {
        Self { pid, handle }
    }
}

impl ThreadExt for Dbg {
    type Item = CONTEXT;

    fn suspend(&self) -> Result<()> {
        Ok(suspend_thread(self.handle)?)
    }

    fn resume(&self) -> Result<()> {
        Ok(resume_thread(self.handle)?)
    }

    fn set_context(&self, context: Self::Item) -> Result<()> {
        Ok(set_thread_context(self.handle, &context)?)
    }

    fn get_context(&self) -> Result<Self::Item> {
        Ok(get_thread_context(self.handle)?)
    }
}

impl DebugExt for Dbg {
    type Item = DEBUG_EVENT;

    fn attch(&self) -> Result<()> {
        debug_active_process(self.pid)?;
        debug_set_process_kill_on_exit(0)?;
        Ok(())
    }

    fn wait(&self) -> Result<Self::Item> {
        Ok(wait_for_debug_event(INFINITE)?)
    }

    fn cont(&self, event: Self::Item) -> Result<()> {
        Ok(continue_debug_event(event, true)?)
    }

    fn flush(&self) -> Result<()> {
        Ok(flush_instruction_cache(self.handle)?)
    }
}

pub fn find_ptr<T>(td: T, cond: Cond, size: Size, addr: usize) -> Result<()>
where
    T: DebugExt<Item = DEBUG_EVENT> + ThreadExt<Item = CONTEXT>,
{
    td.suspend()?;
    let mut context = td.get_context()?;
    context.Dr0 = addr as _;
    context.Dr6 = 0x00000000ffff0ff0;
    context.Dr7 = 0x00000000000d0401;
    td.set_context(context)?;
    td.resume()?;

    td.attch()?;
    let event = td.wait()?;
    if event.dwDebugEventCode == EXCEPTION_DEBUG_EVENT {
        let exc = unsafe { event.u.Exception };
        if exc.ExceptionRecord.ExceptionCode == EXCEPTION_SINGLE_STEP {
            let addr = exc.ExceptionRecord.ExceptionAddress as usize;
            println!("addr: 0x{:x}", addr);
        }
        td.flush()?;
    }
    td.cont(event)?;

    Ok(())
}