mod proc64;

pub use self::proc64::{ShareMode, UserTag};
use super::{Error, Pid, ProcessInfo, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

pub trait VirtualQueryExt {
    fn user_tag(&self) -> u32;
    fn share_mode(&self) -> u8;
}

pub trait ProcessInfoExt {
    fn task(&self) -> u32;
}

pub use proc64::{Mapping, Process};
