#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::prelude::FileExt;
use std::{collections::BTreeMap, fs::File, io, mem, ops::Bound::Included, path::Path};

use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::*;
#[cfg(target_os = "windows")]
use crate::file::*;
use crate::ptrs::pointer_chain_scanner;

pub struct Params<'a, W> {
    pub depth: usize,
    pub target: usize,
    pub node: usize,
    pub offset: (usize, usize),
    pub writer: &'a mut W,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Module {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

#[derive(Default)]
pub struct PtrsxScanner {
    pub modules: Vec<Module>,
    pub forward: BTreeMap<usize, usize>,
    pub reverse: BTreeMap<usize, Vec<usize>>,
}

impl PtrsxScanner {
    pub fn create_pointer_map<P>(&mut self, proc: &P, is_align: bool) -> Result<(), Error>
    where
        P: ProcessInfo + VirtualMemoryRead,
    {
        let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
        let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();
        let mut iter = pages.iter().flat_map(|m| {
            let path = Path::new(m.name()?);
            let name = path.has_root().then_some(path)?.to_str()?.to_string();
            Some(Module { start: m.start(), end: m.end(), name })
        });
        let mut current = iter.next().ok_or("no base address module available.")?;
        for page in iter {
            if page.name == current.name {
                current.end = page.end;
            } else {
                self.modules.push(current);
                current = page;
            }
        }
        self.modules.push(current);

        self.forward = create_pointer_map(proc, &region, is_align)?;
        for (&k, &v) in &self.forward {
            self.reverse.entry(v).or_default().push(k);
        }

        Ok(())
    }

    pub fn create_pointer_map_file<W: io::Write>(&self, writer: &mut W, pid: Pid, is_align: bool) -> Result<(), Error> {
        let proc = Process::open(pid)?;
        let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
        let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();

        let mut modules = Vec::with_capacity(pages.len());
        let mut iter = pages.iter().flat_map(|m| {
            let path = Path::new(m.name()?);
            let name = path.has_root().then_some(path)?.to_str()?.to_string();
            Some(Module { start: m.start(), end: m.end(), name })
        });
        let mut current = iter.next().ok_or("no base address module available.")?;
        for page in iter {
            if page.name == current.name {
                current.end = page.end;
            } else {
                modules.push(current);
                current = page;
            }
        }
        modules.push(current);
        encode_modules(&modules, writer)?;
        create_pointer_map_with_writer(&proc, &region, is_align, writer)
    }

    pub fn load_pointer_map_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let file = File::open(&path)?;
        const SIZE: usize = 8 + mem::size_of::<usize>();
        let mut headers = [0; SIZE];
        let mut seek = 0_u64;
        file.read_exact_at(&mut headers, seek)?;
        seek += headers.len() as u64;

        let (_, len) = headers.split_at(8);
        let len = usize::from_le_bytes(unsafe { *(len.as_ptr().cast()) });

        let mut buf = vec![0; len];
        file.read_exact_at(&mut buf, seek)?;
        self.modules = decode_modules(&buf);
        seek += len as u64;

        let mut buf = vec![0; PTRSIZE * 0x10000];
        loop {
            let size = file.read_at(&mut buf, seek)?;
            if size == 0 {
                break;
            }
            for chuks in buf[..size].chunks(PTRSIZE * 2) {
                let (key, value) = chuks.split_at(PTRSIZE);
                unsafe {
                    self.forward.insert(
                        usize::from_le_bytes(*(key.as_ptr().cast())),
                        usize::from_le_bytes(*(value.as_ptr().cast())),
                    )
                };
            }
            seek += size as u64;
        }

        self.forward.iter().for_each(|(&k, &v)| {
            self.reverse.entry(v).or_default().push(k);
        });

        Ok(())
    }

    pub fn scanner_with_module<W: io::Write>(&self, module: &Module, params: Params<W>) -> io::Result<()> {
        let points = &self
            .forward
            .range((Included(module.start), Included(module.end)))
            .map(|(&k, _)| k)
            .collect::<Vec<_>>();

        let super::Params { depth, target, node, offset, writer } = params;
        let params = ptrs::Params { base: module.start, depth, target, node, offset, points, writer };
        pointer_chain_scanner(&self.reverse, params)
    }

    pub fn scanner_with_address<W: io::Write>(&self, address: usize, params: Params<W>) -> io::Result<()> {
        let super::Params { depth, target, node, offset, writer } = params;
        let params = ptrs::Params { base: 0, depth, target, node, offset, points: &[address], writer };
        pointer_chain_scanner(&self.reverse, params)
    }
}
