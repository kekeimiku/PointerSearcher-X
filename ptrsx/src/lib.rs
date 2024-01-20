use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    io::{BufReader, BufWriter, Cursor, Read, Write},
    mem,
    ops::{Bound, Range},
    path::Path,
};

use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

const PTRSIZE: usize = mem::size_of::<usize>();
const PAGE_SIZE: usize = 0x100000;

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
    use std::fs::File;

    if page.user_tag() == 31 || page.share_mode() == 3 {
        return false;
    }

    let Some(name) = page.name() else {
        return matches!(page.user_tag(), |1..=9| 11 | 30 | 33 | 60 | 61);
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
    use std::fs::File;

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
    use std::fs::File;

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
fn check_region<Q: VirtualQuery + vmmap::windows::VirtualQueryExt>(page: &Q) -> bool {
    use std::fs::File;

    if page.is_guard() || page.is_free() {
        return false;
    }

    let Some(name) = page.name() else {
        return true;
    };
    if name[..40].contains("\\Windows\\") {
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

struct Module<'a> {
    start: usize,
    end: usize,
    name: &'a str,
}

struct ModuleIter<'a>(core::str::Lines<'a>);

impl<'a> ModuleIter<'a> {
    fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = Module<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(2, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let name = split.next()?.trim();
        Some(Module { start, end, name })
    }
}

struct ChunkIter {
    max: usize,
    size: usize,
    pos: usize,
}

impl ChunkIter {
    #[inline]
    fn new(max: usize, size: usize) -> Self {
        Self { max, size, pos: 0 }
    }
}

impl Iterator for ChunkIter {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.max {
            None
        } else {
            let curr = self.pos;
            self.pos = (self.pos + self.size).min(self.max);
            Some((curr, self.pos - curr))
        }
    }
}

#[inline]
fn is_pointer<V: VirtualQuery>(v: &usize, vqs: &[V]) -> bool {
    vqs.binary_search_by(|vq| {
        let (start, size) = (vq.start(), vq.size());
        match (start..start + size).contains(v) {
            true => Ordering::Equal,
            false => start.cmp(v),
        }
    })
    .is_ok()
}

fn create_pointer_map_is_align<P, V, W>(proc: &P, vqs: &[V], w: &mut W) -> Result<(), Error>
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    W: Write,
{
    let mut buf = vec![0; PAGE_SIZE];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, PAGE_SIZE) {
            if proc.read_exact_at(&mut buf[..size], start + off).is_err() {
                break;
            };
            for (k, v) in buf[..size]
                .windows(PTRSIZE)
                .enumerate()
                .step_by(PTRSIZE)
                .map(|(k, v)| (k, usize::from_le_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                let k = start + off + k;
                w.write_all(&k.to_le_bytes())?;
                w.write_all(&v.to_le_bytes())?;
            }
        }
    }
    Ok(())
}

fn create_pointer_map_no_align<P, V, W>(proc: &P, vqs: &[V], w: &mut W) -> Result<(), Error>
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    W: Write,
{
    let mut buf = vec![0; PAGE_SIZE];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, PAGE_SIZE) {
            proc.read_exact_at(&mut buf[..size], start + off)?;
            for (k, v) in buf[..size]
                .windows(PTRSIZE)
                .enumerate()
                .map(|(k, v)| (k, usize::from_le_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                let k = start + off + k;
                w.write_all(&k.to_le_bytes())?;
                w.write_all(&v.to_le_bytes())?;
            }
        }
    }
    Ok(())
}

#[derive(Default)]
enum ScanMode {
    #[default]
    Dense,
    Sparse,
}

#[derive(Default)]
pub struct PtrsxScanner {
    index: RangeMap<usize, String>,
    forward: BTreeSet<usize>,
    reverse: BTreeMap<usize, Vec<usize>>,
    mode: ScanMode,
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
        let vqs = proc
            .get_maps()
            .flatten()
            .filter(|x| x.is_read())
            .filter(check_region)
            .collect::<Vec<_>>();
        let mut counts = HashMap::new();
        let mut info_w = BufWriter::new(info_w);
        vqs.iter()
            .flat_map(|m| {
                let name = Path::new(m.name()?).file_name().and_then(|s| s.to_str())?;
                let count = counts.entry(name).or_insert(0);
                let name = format!("{name}[{count}]");
                *count += 1;
                Some((m.start(), m.end(), name))
            })
            .try_for_each(|(start, end, name)| writeln!(info_w, "{start:x}-{end:x} {name}"))?;
        match align {
            true => create_pointer_map_is_align(&proc, &vqs, &mut BufWriter::new(bin_w)),
            false => create_pointer_map_no_align(&proc, &vqs, &mut BufWriter::new(bin_w)),
        }
    }

    pub fn load_pointer_map<R: Read>(&mut self, reader: R) -> Result<()> {
        const BUF_SIZE: usize = PTRSIZE * 0x20000;
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

        let count = self.reverse.values().filter(|v| v.len() < 64).count();
        self.mode = match (self.reverse.len() - count).checked_mul(512) {
            Some(n) if n < count => ScanMode::Sparse,
            _ => ScanMode::Dense,
        };

        Ok(())
    }

    pub fn load_modules_info<R: Read>(&mut self, r: R) -> Result<()> {
        let contents = &mut String::with_capacity(0x80000);
        let mut reader = BufReader::new(r);
        let _ = reader.read_to_string(contents)?;
        self.index
            .extend(ModuleIter::new(contents).map(|Module { start, end, name }| (start..end, name.to_string())));
        Ok(())
    }

    pub fn pointer_chain_scanner<W: Write>(&self, param: Param, w: W) -> Result<()> {
        let points = &self
            .index
            .iter()
            .flat_map(|(Range { start, end }, _)| self.forward.range((Bound::Included(start), Bound::Included(end))))
            .copied()
            .collect::<Vec<_>>();
        match self.mode {
            ScanMode::Dense => {
                self.scanner_dense(param, points, 1, &mut Vec::with_capacity(0x200), &mut BufWriter::new(w))
            }
            ScanMode::Sparse => {
                self.scanner_sparse(param, points, 1, &mut Vec::with_capacity(0x200), &mut BufWriter::new(w))
            }
        }
    }

    fn scanner_dense<W>(
        &self,
        param: Param,
        points: &[usize],
        lv: usize,
        chain: &mut Vec<isize>,
        w: &mut W,
    ) -> Result<()>
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
                list.iter().try_for_each(|&addr| {
                    self.scanner_dense(Param { depth, addr, node, range }, points, lv + 1, chain, w)
                })?;
                chain.pop();
            }
        }

        Ok(())
    }

    fn scanner_sparse<W>(
        &self,
        param: Param,
        points: &[usize],
        lv: usize,
        chain: &mut Vec<isize>,
        w: &mut W,
    ) -> Result<()>
    where
        W: Write,
    {
        let Param { depth, addr, node, range } = param;

        let min = addr.saturating_sub(range.1);
        let max = addr.saturating_add(range.0);

        let idx = points.iter().position(|x| min.le(x)).unwrap_or(points.len());

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
                list.iter().try_for_each(|&addr| {
                    self.scanner_sparse(Param { depth, addr, node, range }, points, lv + 1, chain, w)
                })?;
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
        Ok(ModuleIter::new(contents)
            .map(|Module { start, end, name }| (start..end, name.to_string()))
            .collect())
    }
}
