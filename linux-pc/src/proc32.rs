use std::{
    fs,
    fs::File,
    io,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

pub struct Process32 {
    pid: i32,
    pathname: PathBuf,
    maps: String,
    handle: File,
}

impl Process32 {
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.read_at(buf, offset)
    }

    pub fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<()> {
        self.handle.write_at(buf, offset).map(drop)
    }
}

impl Process32 {
    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn app_path(&self) -> &Path {
        &self.pathname
    }

    pub fn get_maps(&self) -> impl Iterator<Item = Page32> + '_ {
        Page32Iter::new(&self.maps)
    }
}

impl Process32 {
    pub fn open(pid: i32) -> io::Result<Self> {
        let maps = fs::read_to_string(format!("/proc/{pid}/maps"))?;
        let pathname = fs::read_link(format!("/proc/{pid}/exe"))?;
        let handle = File::open(format!("/proc/{pid}/mem"))?;
        Ok(Self { pid, pathname, maps, handle })
    }
}

#[allow(dead_code)]
pub struct Page32<'a> {
    start: u32,
    end: u32,
    flags: &'a str,
    offset: u32,
    dev: &'a str,
    inode: u32,
    pathname: &'a str,
}

impl Page32<'_> {
    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.end
    }

    pub fn size(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }

    pub fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }

    pub fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }

    pub fn name(&self) -> &str {
        self.pathname
    }
}

pub struct Page32Iter<'a>(core::str::Lines<'a>);

impl<'a> Page32Iter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for Page32Iter<'a> {
    type Item = Page32<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(6, ' ');
        let mut range_split = split.next()?.split('-');
        let start = u32::from_str_radix(range_split.next()?, 16).ok()?;
        let end = u32::from_str_radix(range_split.next()?, 16).ok()?;
        let flags = split.next()?;
        let offset = u32::from_str_radix(split.next()?, 16).ok()?;
        let dev = split.next()?;
        let inode = split.next()?.parse::<u32>().ok()?;
        let pathname = split.next()?.trim_start();

        Some(Page32 { start, end, flags, offset, dev, inode, pathname })
    }
}

#[test]
fn test_parse_maps_32() {
    let maps = r#"56651000-56652000 r--p 00000000 08:02 2233115                            /home/kk/main
56652000-56653000 r-xp 00001000 08:02 2233115                            /home/kk/main
56653000-56654000 r--p 00002000 08:02 2233115                            /home/kk/main
56654000-56655000 r--p 00002000 08:02 2233115                            /home/kk/main
56655000-56656000 rw-p 00003000 08:02 2233115                            /home/kk/main
567d6000-567f8000 rw-p 00000000 00:00 0                                  [heap]
f7c00000-f7c1e000 r--p 00000000 08:02 2373476                            /usr/lib32/libc.so.6
f7c1e000-f7da2000 r-xp 0001e000 08:02 2373476                            /usr/lib32/libc.so.6
f7da2000-f7e21000 r--p 001a2000 08:02 2373476                            /usr/lib32/libc.so.6
f7e21000-f7e23000 r--p 00220000 08:02 2373476                            /usr/lib32/libc.so.6
f7e23000-f7e24000 rw-p 00222000 08:02 2373476                            /usr/lib32/libc.so.6
f7e24000-f7e2e000 rw-p 00000000 00:00 0 
f7ef3000-f7ef5000 rw-p 00000000 00:00 0 
f7ef5000-f7ef9000 r--p 00000000 00:00 0                                  [vvar]
f7ef9000-f7efb000 r-xp 00000000 00:00 0                                  [vdso]
f7efb000-f7efc000 r--p 00000000 08:02 2366679                            /usr/lib32/ld-linux.so.2
f7efc000-f7f1f000 r-xp 00001000 08:02 2366679                            /usr/lib32/ld-linux.so.2
f7f1f000-f7f2d000 r--p 00024000 08:02 2366679                            /usr/lib32/ld-linux.so.2
f7f2d000-f7f2f000 r--p 00031000 08:02 2366679                            /usr/lib32/ld-linux.so.2
f7f2f000-f7f30000 rw-p 00033000 08:02 2366679                            /usr/lib32/ld-linux.so.2
ffeac000-ffecd000 rw-p 00000000 00:00 0                                  [stack]"#;

    assert_eq!(Page32Iter::new(maps).count(), 21)
}
