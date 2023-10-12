mod proc64;

pub use proc64::{Page, Process};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait ProcessInfoExt {
    fn handle(&self) -> isize;
}
