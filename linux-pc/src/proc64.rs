use std::{
    fs,
    fs::File,
    io,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

pub struct Process64 {
    pid: i32,
    pathname: PathBuf,
    maps: String,
    handle: File,
}

impl Process64 {
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.read_at(buf, offset)
    }

    pub fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<()> {
        self.handle.write_at(buf, offset).map(drop)
    }
}

impl Process64 {
    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn app_path(&self) -> &Path {
        &self.pathname
    }

    pub fn get_maps(&self) -> impl Iterator<Item = Page64> + '_ {
        Page64Iter::new(&self.maps)
    }
}

impl Process64 {
    pub fn open(pid: i32) -> io::Result<Self> {
        let maps = fs::read_to_string(format!("/proc/{pid}/maps"))?;
        let pathname = fs::read_link(format!("/proc/{pid}/exe"))?;
        let handle = File::open(format!("/proc/{pid}/mem"))?;
        Ok(Self { pid, pathname, maps, handle })
    }
}

#[allow(dead_code)]
pub struct Page64<'a> {
    start: u64,
    end: u64,
    flags: &'a str,
    offset: u64,
    dev: &'a str,
    inode: u64,
    pathname: &'a str,
}

impl Page64<'_> {
    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn size(&self) -> u64 {
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

pub struct Page64Iter<'a>(core::str::Lines<'a>);

impl<'a> Page64Iter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for Page64Iter<'a> {
    type Item = Page64<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(6, ' ');
        let mut range_split = split.next()?.split('-');
        let start = u64::from_str_radix(range_split.next()?, 16).ok()?;
        let end = u64::from_str_radix(range_split.next()?, 16).ok()?;
        let flags = split.next()?;
        let offset = u64::from_str_radix(split.next()?, 16).ok()?;
        let dev = split.next()?;
        let inode = split.next()?.parse::<u64>().ok()?;
        let pathname = split.next()?.trim_start();

        Some(Page64 { start, end, flags, offset, dev, inode, pathname })
    }
}

#[test]
fn test_parse_maps_64() {
    let maps = r#"5555565cf000-5555565d0000 ---p 00000000 00:00 0                          [heap]
5555565d0000-5555565d1000 rw-p 00000000 00:00 0                          [heap]
7f1cc09a0000-7f1cc09a9000 rw-p 00000000 00:00 0 
7f1cc09a9000-7f1cc09aa000 ---p 00000000 00:00 0 
7f1cc09aa000-7f1cc09ac000 rw-p 00000000 00:00 0 
7f1cc09ac000-7f1cc09ad000 r--p 00000000 08:02 2233120                    /home/kk/hello
7f1cc09ad000-7f1cc09b6000 r-xp 00001000 08:02 2233120                    /home/kk/hello
7f1cc09b6000-7f1cc09b8000 r--p 0000a000 08:02 2233120                    /home/kk/hello
7f1cc09b8000-7f1cc09ba000 rw-p 0000b000 08:02 2233120                    /home/kk/hello
7f1cc09ba000-7f1cc09bb000 rw-p 00000000 00:00 0 
7fff77f9b000-7fff77fbc000 rw-p 00000000 00:00 0                          [stack]
7fff77fde000-7fff77fe2000 r--p 00000000 00:00 0                          [vvar]
7fff77fe2000-7fff77fe4000 r-xp 00000000 00:00 0                          [vdso]
ffffffffff600000-ffffffffff601000 --xp 00000000 00:00 0                  [vsyscall]"#;

    assert_eq!(Page64Iter::new(maps).count(), 14)
}
