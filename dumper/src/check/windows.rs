use consts::EXE;
use vmmap::{windows::VirtualQueryExt, VirtualQuery};

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(map: &Q) -> bool {
    if !map.is_read() {
        return false;
    }
    if map.path().is_none() || check_exe(map) {
        return true;
    }
    false
}

#[inline]
fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    let Some(path) = map.path() else {
        return false;
    };

    if path.starts_with("\\Device\\HarddiskVolume3\\Windows\\System32") {
        return false;
    }

    if path
        .extension()
        .and_then(|s| s.to_str())
        .map_or(false, |n| EXE.contains(&n))
    {
        return true;
    }
    false
}
