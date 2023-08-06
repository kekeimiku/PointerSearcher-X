mod proc64;

use std::path::Path;

pub use proc64::{Page, Process};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn path(&self) -> Option<&Path>;
}

pub trait ProcessInfoExt {
    fn handle(&self) -> isize;
}
