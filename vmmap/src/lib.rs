#![feature(return_position_impl_trait_in_trait)]

#[cfg(all(target_os = "macos", target_pointer_width = "32"))]
panic!("32-bit macos is not supported.");

mod error;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::path::Path;

pub use self::error::Error;
#[cfg(target_os = "linux")]
pub use self::linux::{Page, Process};
#[cfg(target_os = "macos")]
pub use self::macos::{Page, Process};
#[cfg(target_os = "windows")]
pub use self::windows::{Page, Process};

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub type Pid = i32;

#[cfg(target_os = "windows")]
pub type Pid = u32;

pub trait VirtualMemoryRead {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error>;
}

pub trait VirtualMemoryWrite {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<(), Error>;
}

pub trait VirtualQuery {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn size(&self) -> usize;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
    fn name(&self) -> Option<&str>;
}

pub trait ProcessInfo {
    fn pid(&self) -> Pid;
    fn app_path(&self) -> &Path;
    fn get_maps(&self) -> impl Iterator<Item = Page>;
}
