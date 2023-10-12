#[cfg(target_os = "windows")]
use std::os::windows::prelude::{FileExt, MetadataExt};

#[cfg(target_os = "windows")]
pub trait WindowsFileExt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()>;
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
}
#[cfg(target_os = "windows")]
impl WindowsFileExt for std::fs::File {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        let size = FileExt::seek_read(self, buf, offset)?;
        if size < buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "failed to write whole buffer"));
        }
        Ok(())
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        FileExt::seek_read(self, buf, offset)
    }
}
#[cfg(target_os = "windows")]
pub trait WindowsMetadataExt {
    fn size(&self) -> u64;
}
#[cfg(target_os = "windows")]
impl WindowsMetadataExt for std::fs::Metadata {
    fn size(&self) -> u64 {
        MetadataExt::file_size(self)
    }
}
