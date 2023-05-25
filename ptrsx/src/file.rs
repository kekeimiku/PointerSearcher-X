pub trait FileExt {
    type Error: std::error::Error;

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize, Self::Error>;
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<(), Self::Error>;
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize, Self::Error>;
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<(), Self::Error>;
}

#[cfg(target_family = "unix")]
impl FileExt for std::fs::File {
    type Error = std::io::Error;

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize, Self::Error> {
        std::os::unix::fs::FileExt::read_at(self, buf, offset)
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<(), Self::Error> {
        std::os::unix::fs::FileExt::read_exact_at(self, buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize, Self::Error> {
        std::os::unix::fs::FileExt::write_at(self, buf, offset)
    }

    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<(), Self::Error> {
        std::os::unix::fs::FileExt::write_all_at(self, buf, offset)
    }
}

#[cfg(target_family = "windows")]
impl FileExt for std::fs::File {
    type Error = std::io::Error;

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize, Self::Error> {
        std::os::windows::fs::FileExt::seek_read(self, buf, offset)
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<(), Self::Error> {
        let size = std::os::windows::fs::FileExt::seek_read(self, buf, offset)?;
        if size < buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"));
        }
        Ok(())
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize, Self::Error> {
        std::os::windows::fs::FileExt::seek_write(self, buf, offset)
    }

    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<(), Self::Error> {
        let size = std::os::windows::fs::FileExt::seek_write(self, buf, offset)?;
        if size < buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "failed to write whole buffer"));
        }
        Ok(())
    }
}

pub trait MetadataExt {
    fn size(&self) -> u64;
}

#[cfg(target_family = "unix")]
impl MetadataExt for std::fs::Metadata {
    fn size(&self) -> u64 {
        std::os::unix::fs::MetadataExt::size(self)
    }
}

#[cfg(target_family = "windows")]
impl MetadataExt for std::fs::Metadata {
    fn size(&self) -> u64 {
        std::os::windows::fs::MetadataExt::file_size(self)
    }
}
