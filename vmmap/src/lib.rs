mod error;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub mod snapshot;

pub use self::error::Error;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::{Mapping, Process};
#[cfg(target_os = "macos")]
pub use self::macos::{Mapping, Process};
#[cfg(target_os = "windows")]
pub use self::windows::{Mapping, Process};

#[cfg(target_family = "unix")]
pub type Pid = i32;

#[cfg(target_os = "windows")]
pub type Pid = u32;

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait VirtualMemoryRead {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize>;
    fn read_exact_at(&self, buf: &mut [u8], offset: usize) -> Result<()>;
}

pub trait VirtualMemoryWrite {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<usize>;
    fn write_all_at(&self, buf: &[u8], offset: usize) -> Result<()>;
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
    fn app_path(&self) -> &std::path::Path;
    fn get_maps(&self) -> impl Iterator<Item = Result<Mapping>>;
}
