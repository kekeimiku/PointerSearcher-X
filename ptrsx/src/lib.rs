use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::File,
    io::{BufReader, BufWriter, Cursor, Read, Write},
    mem,
    ops::{Bound, Range},
    path::Path,
    str::Lines,
};

use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

const PTRSIZE: usize = mem::size_of::<usize>();

pub enum Error {
    Vm(vmmap::Error),
    Io(std::io::Error),
}

impl From<vmmap::Error> for Error {
    fn from(value: vmmap::Error) -> Self {
        Self::Vm(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Vm(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(target_os = "macos")]
#[inline]
fn check_region<Q: VirtualQuery + vmmap::macos::VirtualQueryExt>(page: &Q) -> bool {
    if !page.is_read() || page.is_reserve() {
        return false;
    }

    let Some(name) = page.name() else {
        return matches!(page.tag(), |1..=9| 11 | 30 | 33 | 60 | 61);
    };
    let path = Path::new(name);
    if path.starts_with("/usr") {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| match buf[0..4] {
            [width, 0xfa, 0xed, 0xfe] if width == 0xcf || width == 0xce => true,
            [0xfe, 0xed, 0xfa, width] if width == 0xcf || width == 0xce => true,
            [0xca, 0xfe, 0xba, 0xbe] => u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) < 45,
            _ => false,
        })
}

#[cfg(target_os = "linux")]
#[inline]
fn check_region<Q: VirtualQuery>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    let Some(name) = page.name() else {
        return true;
    };
    if name.eq("[stack]") || name.eq("[heap]") {
        return true;
    }
    if name.get(0..7).is_some_and(|s| s.eq("/memfd:")) {
        return false;
    }
    let path = Path::new(name);
    if !path.has_root() || path.starts_with("/dev") {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| [0x7f, b'E', b'L', b'F'].eq(&buf[0..4]))
}

#[cfg(target_os = "android")]
#[inline]
fn check_region<Q: VirtualQuery>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    // anonmyous return true
    let Some(name) = page.name() else {
        return true;
    };

    if name.eq("[anon:.bss]") || name.eq("[anon:libc_malloc]") || name.eq("[stack]") || name.eq("[heap]") {
        return true;
    }

    if name.get(0..7).is_some_and(|s| s.eq("/memfd:")) {
        return false;
    }

    let path = Path::new(name);

    if !path.has_root()
        || path.starts_with("/dev")
        || path.starts_with("/system")
        || path.starts_with("/system_ext")
        || path.starts_with("/apex")
        || path.starts_with("/product")
        || path.starts_with("/vendor")
        || path.extension().is_some_and(|x| x.eq("dex") || x.eq("odex"))
    {
        return false;
    }

    let mut buf = [0; 64];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| [0x7f, b'E', b'L', b'F'].eq(&buf[0..4]))
}

#[cfg(target_os = "windows")]
#[inline]
fn check_region<Q: VirtualQuery>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    let Some(name) = page.name() else {
        return true;
    };
    if name.contains("\\Windows\\System32\\") {
        return false;
    }
    let name = name.replacen(r#"\Device"#, r#"\\?"#, 1);
    let path = Path::new(&name);
    if !path.has_root() {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| [0x4d, 0x5a].eq(&buf[0..2]))
}

struct RangeWrapper<T>(Range<T>);

impl<T: PartialEq> PartialEq for RangeWrapper<T> {
    fn eq(&self, other: &RangeWrapper<T>) -> bool {
        self.0.start == other.0.start
    }
}

impl<T: Eq> Eq for RangeWrapper<T> {}

impl<T: Ord> Ord for RangeWrapper<T> {
    fn cmp(&self, other: &RangeWrapper<T>) -> Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl<T: PartialOrd> PartialOrd for RangeWrapper<T> {
    fn partial_cmp(&self, other: &RangeWrapper<T>) -> Option<Ordering> {
        self.0.start.partial_cmp(&other.0.start)
    }
}

#[derive(Default)]
struct RangeMap<K, V>(BTreeMap<RangeWrapper<K>, V>);

impl<K, V> RangeMap<K, V> {
    fn iter(&self) -> impl Iterator<Item = (&Range<K>, &V)> {
        self.0.iter().map(|(k, v)| (&k.0, v))
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl<K, V> RangeMap<K, V>
where
    K: Ord + Copy,
{
    fn get_key_value(&self, point: K) -> Option<(&Range<K>, &V)> {
        let start = RangeWrapper(point..point);
        self.0
            .range((Bound::Unbounded, Bound::Included(start)))
            .next_back()
            .filter(|(range, _)| range.0.contains(&point))
            .map(|(range, value)| (&range.0, value))
    }

    fn insert(&mut self, key: Range<K>, value: V) -> Option<V> {
        assert!(key.start <= key.end);
        self.0.insert(RangeWrapper(key), value)
    }
}

impl<K, V> Extend<(Range<K>, V)> for RangeMap<K, V>
where
    K: Ord + Copy,
{
    fn extend<T: IntoIterator<Item = (Range<K>, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.insert(k, v);
        })
    }
}

struct Info<'a> {
    start: usize,
    end: usize,
    name: &'a str,
}

struct InfoIter<'a>(Lines<'a>);

impl<'a> InfoIter<'a> {
    fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for InfoIter<'a> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(2, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let name = split.next()?.trim();
        Some(Info { start, end, name })
    }
}

fn create_pointer_map<P, W>(proc: &P, region: &[(usize, usize)], is_align: bool, w: &mut W) -> Result<(), Error>
where
    P: VirtualMemoryRead,
    W: Write,
{
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    const BUF_SIZE: usize = 0x4000;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const BUF_SIZE: usize = 0x40000;

    #[cfg(any(target_os = "windows", all(target_os = "macos", target_arch = "x86_64"),))]
    const BUF_SIZE: usize = 0x1000;

    let mut buf = [0; BUF_SIZE];

    if is_align {
        for &(start, size) in region {
            for off in (0..size).step_by(BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, value) in buf[..size]
                    .windows(PTRSIZE)
                    .enumerate()
                    .step_by(PTRSIZE)
                    .map(|(k, v)| (k, usize::from_le_bytes(v.try_into().unwrap())))
                {
                    if region
                        .binary_search_by(|&(start, size)| {
                            if (start..start + size).contains(&value) {
                                Ordering::Equal
                            } else {
                                start.cmp(&value)
                            }
                        })
                        .is_ok()
                    {
                        let key = start + off + k;
                        w.write_all(&key.to_le_bytes())?;
                        w.write_all(&value.to_le_bytes())?;
                    }
                }
            }
        }
    } else {
        for &(start, size) in region {
            for off in (0..size).step_by(BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, value) in buf[..size]
                    .windows(PTRSIZE)
                    .enumerate()
                    .map(|(k, v)| (k, usize::from_le_bytes(v.try_into().unwrap())))
                {
                    if region
                        .binary_search_by(|&(start, size)| {
                            if (start..start + size).contains(&value) {
                                Ordering::Equal
                            } else {
                                start.cmp(&value)
                            }
                        })
                        .is_ok()
                    {
                        let key = start + off + k;
                        w.write_all(&key.to_le_bytes())?;
                        w.write_all(&value.to_le_bytes())?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct PtrsxScanner {
    index: RangeMap<usize, String>,
    forward: BTreeSet<usize>,
    reverse: BTreeMap<usize, Vec<usize>>,
}

pub struct Param {
    pub depth: usize,
    pub addr: usize,
    pub node: usize,
    pub range: (usize, usize),
}

impl PtrsxScanner {
    pub fn create_pointer_map<W: Write>(&self, pid: Pid, align: bool, info_w: W, bin_w: W) -> Result<()> {
        let proc = Process::open(pid)?;
        let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
        let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();
        let mut counts = HashMap::new();
        let mut info_w = BufWriter::new(info_w);
        pages
            .iter()
            .flat_map(|m| {
                let name = Path::new(m.name()?).file_name().and_then(|s| s.to_str())?;
                let count = counts.entry(name).or_insert(0);
                let name = format!("{name}[{count}]");
                *count += 1;
                Some((m.start(), m.end(), name))
            })
            .try_for_each(|(start, end, name)| writeln!(info_w, "{start:x}-{end:x} {name}"))?;
        create_pointer_map(&proc, &region, align, &mut BufWriter::new(bin_w))
    }

    pub fn load_pointer_map<R: Read>(&mut self, reader: R) -> Result<()> {
        const BUF_SIZE: usize = PTRSIZE * 0x10000;
        const CHUNK_SIZE: usize = PTRSIZE * 2;
        let mut buf = vec![0; BUF_SIZE];
        let mut cursor = Cursor::new(reader);
        loop {
            let size = cursor.get_mut().read(&mut buf)?;
            if size == 0 {
                break;
            }
            for chuks in buf[..size].chunks_exact(CHUNK_SIZE) {
                let (key, value) = chuks.split_at(PTRSIZE);
                let (key, value) =
                    (usize::from_le_bytes(key.try_into().unwrap()), usize::from_le_bytes(value.try_into().unwrap()));
                if self.forward.insert(key) {
                    self.reverse.entry(value).or_default().push(key);
                }
            }
        }
        Ok(())
    }

    pub fn load_modules_info<R: Read>(&mut self, r: R) -> Result<()> {
        let contents = &mut String::with_capacity(0x80000);
        let mut reader = BufReader::new(r);
        let _ = reader.read_to_string(contents)?;
        self.index
            .extend(InfoIter::new(contents).map(|Info { start, end, name }| (start..end, name.to_string())));
        Ok(())
    }

    pub fn pointer_chain_scanner<W: Write>(&self, param: Param, w: W) -> Result<()> {
        let points = &self
            .index
            .iter()
            .flat_map(|(Range { start, end }, _)| self.forward.range((Bound::Included(start), Bound::Included(end))))
            .copied()
            .collect::<Vec<_>>();
        self.scanner(param, points, 1, &mut Vec::with_capacity(0x200), &mut BufWriter::new(w))
    }

    fn scanner<W>(&self, param: Param, points: &[usize], lv: usize, chain: &mut Vec<isize>, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        let Param { depth, addr, node, range } = param;

        let min = addr.saturating_sub(range.1);
        let max = addr.saturating_add(range.0);

        let idx = points.binary_search(&min).unwrap_or_else(|x| x);

        if points
            .iter()
            .skip(idx)
            .take_while(|x| max.ge(x))
            .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
            .is_some_and(|_| chain.len() >= node)
        {
            if let Some((Range { start, end: _ }, name)) = self.index.get_key_value(addr) {
                write!(w, "{name}+{}", addr - start)?;
                chain.iter().rev().try_for_each(|o| write!(w, "@{o}"))?;
                writeln!(w)?;
            }
        }

        if lv <= depth {
            for (&k, list) in self.reverse.range((Bound::Included(min), Bound::Included(max))) {
                chain.push(addr.wrapping_sub(k) as isize);
                list.iter()
                    .try_for_each(|&addr| self.scanner(Param { depth, addr, node, range }, points, lv + 1, chain, w))?;
                chain.pop();
            }
        }

        Ok(())
    }

    pub fn set_modules<I>(&mut self, i: I)
    where
        I: Iterator<Item = (Range<usize>, String)>,
    {
        self.index.extend(i)
    }

    pub fn reset(&mut self) {
        self.index.clear();
        self.forward.clear();
        self.reverse.clear();
    }

    pub fn parse_modules_info<R: Read>(&mut self, r: R) -> Result<Vec<(Range<usize>, String)>> {
        let contents = &mut String::with_capacity(0x80000);
        let mut reader = BufReader::new(r);
        let _ = reader.read_to_string(contents)?;
        Ok(InfoIter::new(contents)
            .map(|Info { start, end, name }| (start..end, name.to_string()))
            .collect())
    }
}
