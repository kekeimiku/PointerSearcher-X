use std::io::{BufWriter, Write};

use super::{SnapshotError, VirtualMemoryRead, VirtualQuery};

type Result<T, E = SnapshotError> = core::result::Result<T, E>;

// set append write
pub fn create_snapshot<P, V, I, W>(proc: &P, region: &mut I, info_w: &mut W, bin_w: &mut W) -> Result<()>
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    I: Iterator<Item = V>,
    W: Write,
{
    const DEFAULT_BUF_SIZE: usize = 0x200000;
    let mut info_w = BufWriter::new(info_w);
    let mut bin_w = BufWriter::with_capacity(DEFAULT_BUF_SIZE, bin_w);

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    const BUF_SIZE: usize = 0x4000;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const BUF_SIZE: usize = 0x40000;

    #[cfg(any(target_os = "windows", all(target_os = "macos", target_arch = "x86_64"),))]
    const BUF_SIZE: usize = 0x1000;

    let mut buf = vec![0; BUF_SIZE];

    let mut e_seek = 0;
    for vq in region {
        let mut i_seek = 0;
        let (start, size) = (vq.start(), vq.size());
        for off in (0..size).step_by(BUF_SIZE) {
            let size = proc.read_at(&mut buf, start + off)?;
            bin_w.write_all(&buf[..size])?;
            i_seek += size;
        }

        let remark = vq.name().unwrap_or_default();
        let (real_start, real_end) = (vq.start(), vq.end());
        let (virt_start, virt_end) = (e_seek, e_seek + i_seek);

        writeln!(
            info_w,
            "{real_start:x}-{real_end:x}={virt_start:x}-{virt_end:x} {}{}{} {remark}",
            if vq.is_read() { "r" } else { "-" },
            if vq.is_write() { "w" } else { "-" },
            if vq.is_exec() { "x" } else { "-" },
        )?;

        e_seek += i_seek;
    }

    Ok(())
}

pub struct Page<'a> {
    real_start: usize,
    real_end: usize,
    virt_start: usize,
    virt_end: usize,
    flags: &'a str,
    remark: Option<&'a str>,
}

impl Page<'_> {
    #[inline]
    pub const fn real_start(&self) -> usize {
        self.real_start
    }

    #[inline]
    pub const fn virt_start(&self) -> usize {
        self.virt_start
    }

    #[inline]
    pub const fn real_end(&self) -> usize {
        self.real_end
    }

    #[inline]
    pub const fn virt_end(&self) -> usize {
        self.virt_end
    }

    #[inline]
    pub const fn real_size(&self) -> usize {
        self.real_end - self.real_start
    }

    #[inline]
    pub const fn virt_size(&self) -> usize {
        self.virt_end - self.virt_start
    }

    #[inline]
    pub fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }

    #[inline]
    pub fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }

    #[inline]
    pub fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }

    #[inline]
    pub const fn remark(&self) -> Option<&str> {
        self.remark
    }
}

pub struct Iter<'a>(core::str::Lines<'a>);

impl<'a> Iter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Page<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut whitespace_split = line.splitn(3, ' ');
        let range = whitespace_split.next()?;
        let mut range_split = range.split('=');

        let real_range = range_split.next()?;
        let mut real_range_split = real_range.split('-');
        let real_start = usize::from_str_radix(real_range_split.next()?, 16).ok()?;
        let real_end = usize::from_str_radix(real_range_split.next()?, 16).ok()?;

        let virt_range = range_split.next()?;
        let mut virt_range_split = virt_range.split('-');
        let virt_start = usize::from_str_radix(virt_range_split.next()?, 16).ok()?;
        let virt_end = usize::from_str_radix(virt_range_split.next()?, 16).ok()?;

        let flags = whitespace_split.next()?;
        let remark = whitespace_split.next()?;
        let remark = (!remark.is_empty()).then_some(remark);

        Some(Page { real_start, real_end, virt_start, virt_end, flags, remark })
    }
}
