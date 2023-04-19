use vmmap::{windows::VirtualQueryExt, VirtualQuery};

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(_map: &Q) -> bool {
    todo!()
}
