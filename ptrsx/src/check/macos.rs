use std::{fs::File, io::Read};

use utils::consts::EXE;
use vmmap::{macos::VirtualQueryExt, VirtualQuery};

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(map: &Q) -> bool {
    if !map.is_read() {
        return false;
    }

    if check_exe(map) || map.path().is_none() && matches!(map.tag(), |1..=9| 11 | 30 | 33 | 60 | 61) {
        return true;
    }

    false
}

#[inline]
fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    let Some(path) = map.path() else {
        return false;
    };

    if path.starts_with("/usr") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_or(false, |_| EXE.contains(&header))
}
