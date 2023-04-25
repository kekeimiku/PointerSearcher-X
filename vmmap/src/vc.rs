use core::arch::global_asm;

global_asm! {
    r#".macro SVC_BEGIN name
	.section .text.\name, "ax", %progbits
	.global \name
	.type \name, %function
	.align 2
	.cfi_startproc
\name:
.endm
.macro SVC_END
	.cfi_endproc
.endm
SVC_BEGIN svcGetProcessList
	str x0, [sp, #-16]!
	svc 0x65
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END
SVC_BEGIN svcDebugActiveProcess
	str x0, [sp, #-16]!
	svc 0x60
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END
SVC_BEGIN svcWriteDebugProcessMemory
	svc 0x6B
	ret
SVC_END
SVC_BEGIN svcQueryDebugProcessMemory
	str x1, [sp, #-16]!
	svc 0x69
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END
SVC_BEGIN svcReadDebugProcessMemory
	svc 0x6A
	ret
SVC_END
"#
}

extern "C" {
    fn svcGetProcessList(num_out: *mut u32, pids_out: *mut u64, max_pids: u32) -> u32;
    fn svcDebugActiveProcess(handle: *mut u32, pid: u64) -> u32;
    fn svcWriteDebugProcessMemory(handle: u32, buffer: *const u8, address: usize, size: usize) -> u32;
    fn svcQueryDebugProcessMemory(
        out_info: *mut MemoryInfo,
        out_page_info: *mut u32,
        handle: u32,
        address: usize,
    ) -> u32;
    fn svcReadDebugProcessMemory(buffer: *mut u8, handle: u32, address: usize, size: usize) -> u32;
}

pub type Result<T> = core::result::Result<T, u32>;

#[inline(always)]
pub fn svc_get_process_list(list: &mut [u64]) -> Result<usize> {
    unsafe {
        let mut count: u32 = 0;
        let rc = svcGetProcessList(&mut count, list.as_mut_ptr(), list.len() as u32);
        if rc == 0 {
            Ok(count as usize)
        } else {
            Err(rc)
        }
    }
}

#[inline(always)]
pub fn svc_debug_active_process(pid: u64) -> Result<u32> {
    unsafe {
        let mut handle: u32 = 0;

        let rc = svcDebugActiveProcess(&mut handle, pid);
        if rc == 0 {
            Ok(handle)
        } else {
            Err(rc)
        }
    }
}

#[inline(always)]
pub fn svc_write_debug_process_memory(handle: u32, addr: usize, size: usize, buffer: &[u8]) -> Result<()> {
    unsafe {
        let rc = svcWriteDebugProcessMemory(handle, buffer.as_ptr(), addr, size);
        if rc == 0 {
            Ok(())
        } else {
            Err(rc)
        }
    }
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct MemoryInfo {
    pub addr: usize,
    pub size: usize,
    pub state: u32,
    pub attr: u32,
    pub perm: u32,
    pub ipc_refcount: u32,
    pub device_refcount: u32,
    pub padding: u32,
}

#[inline(always)]
pub fn svc_query_debug_process_memory(handle: u32, address: usize) -> Result<(MemoryInfo, u32)> {
    unsafe {
        let mut memory_info: MemoryInfo = Default::default();
        let mut page_info = 0;

        let rc = svcQueryDebugProcessMemory(&mut memory_info, &mut page_info, handle, address);
        if rc == 0 {
            Ok((memory_info, page_info))
        } else {
            Err(rc)
        }
    }
}

#[inline(always)]
pub fn svc_read_debug_process_memory(handle: u32, addr: usize, size: usize, buffer: &mut [u8]) -> Result<()> {
    unsafe {
        let rc = svcReadDebugProcessMemory(buffer.as_mut_ptr(), handle, addr, size);
        if rc == 0 {
            Ok(())
        } else {
            Err(rc)
        }
    }
}
