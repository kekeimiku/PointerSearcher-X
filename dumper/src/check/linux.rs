use std::{fs::File, io::Read};

use utils::consts::EXE;
use vmmap::{linux::VirtualQueryExt, VirtualQuery};

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(map: &Q) -> bool {
    if !map.is_read() {
        return false;
    }

    if matches!(map.name(), "[stack]" | "[heap]") || check_exe(map) || map.path().is_none() {
        return true;
    }

    false
}

#[inline]
fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    let Some(path) = map.path() else {
        return false;
    };

    if path.starts_with("/dev") || path.starts_with("/usr") {
        return false;
    }

    let mut header = [0; 4];
    if let Ok(mut file) = File::open(path) {
        if file.read_exact(&mut header).is_ok() {
            return EXE.contains(&header);
        }
    }
    false
}
