mod proc64;

pub use proc64::{Mapping, Process};

use super::{Error, Pid, ProcessInfo, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait ProcessInfoExt {
    fn handle(&self) -> isize;
}

pub trait VirtualQueryExt {
    fn is_free(&self) -> bool;
    fn is_guard(&self) -> bool;
    fn m_type(&self) -> u32;
    fn m_state(&self) -> u32;
    fn m_protect(&self) -> u32;
}
