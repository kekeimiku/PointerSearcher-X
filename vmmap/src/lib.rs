#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::proc::Process;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::proc::{Map, Process};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::proc::{Map, Process};

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub type Pid = i32;

#[cfg(target_os = "windows")]
pub type Pid = u32;

mod error;
use std::path::Path;

pub use error::Error;

pub trait VirtualMemoryRead {
    type Error: std::error::Error;

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait VirtualMemoryWrite {
    type Error: std::error::Error;

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<(), Self::Error>;
}

pub trait VirtualQuery {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn size(&self) -> usize;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
    fn path(&self) -> Option<&Path>;
}

pub trait ProcessInfo {
    fn pid(&self) -> Pid;
    fn app_path(&self) -> &Path;
    fn get_maps(&self) -> Box<dyn Iterator<Item = Map> + '_>;
}
