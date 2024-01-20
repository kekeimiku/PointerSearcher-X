#[cfg(target_os = "macos")]
pub mod macos {
    use std::{ffi::OsString, mem, os::unix::ffi::OsStringExt, path::PathBuf, ptr};

    use machx::{
        kern_return::kern_return_t,
        libproc::{proc_listpids, proc_pidpath, PROC_ALL_PIDS, PROC_PIDPATHINFO_MAXSIZE},
    };

    unsafe fn pidpath(pid: i32) -> Result<OsString, kern_return_t> {
        let mut buf = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE as usize - 1);
        let ret = proc_pidpath(pid, buf.as_mut_ptr() as _, buf.capacity() as _);
        if ret <= 0 {
            Err(ret)
        } else {
            buf.set_len(ret as usize);
            Ok(OsString::from_vec(buf))
        }
    }

    unsafe fn listpids() -> Result<Vec<i32>, kern_return_t> {
        let size = proc_listpids(PROC_ALL_PIDS, 0, ptr::null_mut(), 0);
        let cap = size as usize / mem::size_of::<i32>();
        let mut pids = Vec::with_capacity(cap);
        let ret = proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr() as _, size);
        if ret <= 0 {
            Err(ret)
        } else {
            let len = ret as usize / mem::size_of::<i32>();
            pids.set_len(len);
            Ok(pids)
        }
    }

    pub fn get_process_list_iter() -> Result<impl Iterator<Item = (i32, PathBuf)>, kern_return_t> {
        let pids = unsafe { listpids() }?;
        let iter = pids
            .into_iter()
            .flat_map(|id| unsafe { pidpath(id) }.ok().map(|s| (id, PathBuf::from(s))));
        Ok(iter)
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    use std::{fs, io, path::PathBuf};

    pub fn get_process_list_iter() -> Result<impl Iterator<Item = (i32, PathBuf)>, io::Error> {
        let dirs = fs::read_dir("/proc")?;
        let iter = dirs.flatten().flat_map(|f| {
            let id = f.file_name().to_str()?.parse().ok()?;
            let path = fs::read_link(f.path().join("exe")).ok()?;
            Some((id, path))
        });
        Ok(iter)
    }
}

#[cfg(target_os = "windows")]
pub mod windows {
    use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt, path::PathBuf};

    use windows_sys::Win32::{
        Foundation::{CloseHandle, GetLastError, FALSE, MAX_PATH, WIN32_ERROR},
        System::{
            ProcessStatus::EnumProcesses,
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_NATIVE, PROCESS_QUERY_LIMITED_INFORMATION,
            },
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
}
