pub mod proc64;

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn tag(&self) -> u32;
    fn is_reserve(&self) -> bool;
}

pub trait ProcessInfoExt {
    fn task(&self) -> u32;
}

pub use proc64::{Page, Process};
