mod proc;

pub use proc::{Page, Process};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn name(&self) -> &str;
}
