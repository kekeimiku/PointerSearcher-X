use std::{fs::File, io::Read};

use vmmap::VirtualQuery;

use crate::consts::EXE;

fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    if !map.is_read() {
        return false;
    }

    let Some(path) = map.path() else {
    return false;
};

    #[cfg(target_os = "linux")]
    if path.starts_with("/dev") || path.starts_with("/usr") {
        return false;
    }

    #[cfg(target_os = "macos")]
    if path.starts_with("/usr") {
        return false;
    }

    if let Ok(mut file) = File::open(path) {
        let mut buf = [0; 4];
        if file.read_exact(&mut buf).is_ok() {
            return EXE.eq(&buf);
        }
    }
    false
}

pub fn check_region<Q: VirtualQuery>(map: &Q) -> bool {
    #[cfg(target_os = "linux")]
    if matches!(map.name(), "[vvar]" | "[vdso]" | "[vsyscall]") {
        return false;
    }

    if check_exe(map) || map.is_stack() || map.is_heap() || map.path().is_none() {
        return true;
    }

    false
}
