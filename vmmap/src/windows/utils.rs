use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt, path::PathBuf};

use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, FALSE, MAX_PATH, WIN32_ERROR},
    System::{
        ProcessStatus::EnumProcesses,
        Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_NATIVE, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

unsafe fn listpids() -> Result<Vec<u32>, WIN32_ERROR> {
    let mut pids: Vec<u32> = Vec::with_capacity(1024);
    let mut size = 0;
    let cb = (mem::size_of::<u32>() * pids.capacity()) as u32;
    let ret = EnumProcesses(pids.as_mut_ptr(), cb, &mut size);
    if ret == FALSE {
        let err = GetLastError();
        Err(err)
    } else {
        let len = size as usize / mem::size_of::<u32>();
        pids.set_len(len);
        Ok(pids)
    }
}

unsafe fn pidpath(pid: u32) -> Result<OsString, WIN32_ERROR> {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    let mut buf = Vec::with_capacity(MAX_PATH as _);
    let mut size = MAX_PATH;
    let ret = QueryFullProcessImageNameW(handle, PROCESS_NAME_NATIVE, buf.as_mut_ptr(), &mut size);
    if ret == FALSE {
        let err = GetLastError();
        Err(err)
    } else {
        let _ = CloseHandle(handle);
        buf.set_len(size as usize);
        Ok(OsString::from_wide(&buf))
    }
}

pub fn get_process_list_iter() -> Result<impl Iterator<Item = (u32, PathBuf)>, WIN32_ERROR> {
    let pids = unsafe { listpids() }?;
    let iter = pids
        .into_iter()
        .flat_map(|id| unsafe { pidpath(id) }.ok().map(|s| (id, PathBuf::from(s))));
    Ok(iter)
}
