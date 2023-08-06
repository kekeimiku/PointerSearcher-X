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
use self::linux::Page;
#[cfg(target_os = "linux")]
pub use self::linux::Process;
#[cfg(target_os = "macos")]
use self::macos::Page;
#[cfg(target_os = "macos")]
pub use self::macos::Process;
#[cfg(target_os = "windows")]
use self::windows::Page;
#[cfg(target_os = "windows")]
pub use self::windows::Process;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub type Pid = i32;

#[cfg(target_os = "windows")]
pub type Pid = u32;

pub trait VirtualMemoryRead {
    type Error: std::error::Error;

    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize, Self::Error>;
}

pub trait VirtualMemoryWrite {
    type Error: std::error::Error;

    fn write_at(&self, buf: &[u8], offset: usize) -> Result<(), Self::Error>;
}

pub trait VirtualQuery {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn size(&self) -> usize;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
}

pub trait ProcessInfo {
    fn pid(&self) -> Pid;
    fn app_path(&self) -> &Path;
    fn get_maps(&self) -> Box<dyn Iterator<Item = Page> + '_>;
}
