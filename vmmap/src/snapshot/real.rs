use std::{
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
};

use memmap2::MmapMut;

use super::{snap, RangeMap, VirtProcError};

type Result<T, E = VirtProcError> = core::result::Result<T, E>;

// real address mode
pub struct RealMode {
    // key: real_address, value: virt_address
    pub map: RangeMap<usize, usize>,
    pub contents: String,
    pub mmap: MmapMut,
    pub path: PathBuf,
}

impl RealMode {
    const RW_ERR: &'static str = "invalid offset";

    pub fn open<P: AsRef<Path>>(info: P, bin: P) -> Result<Self> {
        let path = bin.as_ref().to_path_buf();
        let (contents, mmap) = Self::_open(info, bin).map_err(VirtProcError::OpenProcess)?;
        let mut map = RangeMap::default();
        for m in snap::Iter::new(&contents) {
            let (real_start, real_end, virt_start) = (m.real_start(), m.real_end(), m.virt_start());
            println!("{real_start} {real_end} {virt_start}");
            map.insert(real_start..real_end, virt_start)
        }
        Ok(Self { map, contents, mmap, path })
    }

    fn _open<P: AsRef<Path>>(info: P, bin: P) -> io::Result<(String, MmapMut)> {
        let contents = fs::read_to_string(info)?;
        let file = fs::OpenOptions::new().read(true).write(true).open(bin)?;
        let mmap = unsafe { MmapMut::map_mut(&file) }?;
        Ok((contents, mmap))
    }

    // convert real address to virtual address.
    #[inline]
    fn conv_address(&self, offset: usize) -> Result<usize> {
        self._conv_address(offset)
            .ok_or(VirtProcError::ReadMemory(Self::RW_ERR))
    }

    #[inline]
    fn _conv_address(&self, offset: usize) -> Option<usize> {
        let (&Range { start, .. }, virt_start) = self.map.get_key_value(offset)?;
        let offset = offset.checked_sub(start)?;
        virt_start.checked_add(offset)
    }

    #[inline]
    pub fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        let offset = self.conv_address(offset)?;
        let data = self
            .mmap
            .get(offset..offset + buf.len())
            .ok_or(VirtProcError::ReadMemory(Self::RW_ERR))?;
        buf.copy_from_slice(data);
        Ok(buf.len())
    }

    #[inline]
    pub fn write_at(&mut self, buf: &[u8], offset: usize) -> Result<usize> {
        let offset = self.conv_address(offset)?;
        self.mmap
            .get_mut(offset..offset + buf.len())
            .ok_or(VirtProcError::WriteMemory(Self::RW_ERR))?
            .copy_from_slice(buf);
        Ok(buf.len())
    }

    pub fn get_maps(&self) -> impl Iterator<Item = RealPage> {
        snap::Iter::new(&self.contents).map(RealPage)
    }

    pub fn app_path(&self) -> &Path {
        &self.path
    }
}

pub struct RealPage<'a>(snap::Page<'a>);

impl RealPage<'_> {
    #[inline]
    pub const fn start(&self) -> usize {
        self.0.real_start()
    }

    #[inline]
    pub const fn end(&self) -> usize {
        self.0.real_end()
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.0.real_size()
    }

    #[inline]
    pub fn is_read(&self) -> bool {
        self.0.is_read()
    }

    #[inline]
    pub fn is_write(&self) -> bool {
        self.0.is_write()
    }

    #[inline]
    pub fn is_exec(&self) -> bool {
        self.0.is_exec()
    }

    #[inline]
    pub const fn name(&self) -> Option<&str> {
        self.0.remark()
    }
}
