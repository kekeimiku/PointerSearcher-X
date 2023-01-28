pub mod error;

pub trait MapExt {
    fn start(&self) -> u64;
    fn end(&self) -> u64;
    fn size(&self) -> u64;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
}

pub trait MemExt {
    fn read_at(&self, addr: usize, size: usize) -> crate::error::Result<Vec<u8>>;
    fn write_at(&self, addr: usize, payload: &[u8]) -> crate::error::Result<usize>;
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
use linux::{Map, MapsIter};

#[cfg(target_os = "linux")]
pub fn vmmap(pid: i32) -> std::io::Result<Vec<Map>> {
    let contents = std::fs::read_to_string(format!("/proc/{pid}/maps"))?;
    Ok(MapsIter::new(&contents).map(Into::into).collect())
}

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
use macos::Map;

#[cfg(target_os = "macos")]
pub fn vmmap(task_id: ffi::mach_port_name_t) -> Vec<Map> {
    use macos::MapIter;

    MapIter::new(task_id).collect()
}

#[cfg(target_os = "windows")]
pub mod windows;
