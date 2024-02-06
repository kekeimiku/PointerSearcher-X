use std::{ffi::OsString, mem, os::unix::ffi::OsStringExt, path::PathBuf, ptr};

use machx::{
    kern_return::kern_return_t,
    libproc::{proc_listpids, proc_pidpath, PROC_ALL_PIDS, PROC_PIDPATHINFO_MAXSIZE},
};

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

pub fn get_process_list_iter() -> Result<impl Iterator<Item = (i32, PathBuf)>, kern_return_t> {
    let pids = unsafe { listpids() }?;
    let iter = pids
        .into_iter()
        .flat_map(|id| unsafe { pidpath(id) }.ok().map(|s| (id, PathBuf::from(s))));
    Ok(iter)
}
