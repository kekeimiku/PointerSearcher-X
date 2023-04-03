pub mod proc;

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn tag(&self) -> u32;
    fn is_reserve(&self) -> bool;
}
