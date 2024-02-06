mod proc;
pub mod utils;

pub use proc::{Mapping, Process};

use super::{Error, Pid, ProcessInfo, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn offset(&self) -> usize;
    fn dev(&self) -> &str;
    fn inode(&self) -> usize;
}
