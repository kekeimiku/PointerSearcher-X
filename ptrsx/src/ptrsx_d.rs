use std::{cmp::Ordering, collections::BTreeMap, io, path::Path};

use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::{check_region, encode_modules, Error, Module, PtrsxScanner, DEFAULT_BUF_SIZE, PTRSIZE};

pub fn create_pointer_map<P>(
    proc: &P,
    region: &[(usize, usize)],
    is_align: bool,
) -> Result<BTreeMap<usize, usize>, Error>
where
    P: VirtualMemoryRead,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];
    let mut map = BTreeMap::new();

    if is_align {
        for &(start, size) in region {
            for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, buf) in buf[..size].windows(PTRSIZE).enumerate().step_by(PTRSIZE) {
                    let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
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
                        map.insert(key, value);
                    }
                }
            }
        }
    } else {
        for &(start, size) in region {
            for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, buf) in buf[..size].windows(PTRSIZE).enumerate() {
                    let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
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
                        map.insert(key, value);
                    }
                }
            }
        }
    }

    Ok(map)
}

pub fn create_pointer_map_with_writer<P, W>(
    proc: &P,
    region: &[(usize, usize)],
    is_align: bool,
    writer: &mut W,
) -> Result<(), Error>
where
    P: VirtualMemoryRead,
    W: io::Write,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];

    if is_align {
        for &(start, size) in region {
            for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, buf) in buf[..size].windows(PTRSIZE).enumerate().step_by(PTRSIZE) {
                    let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
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
                        writer.write_all(&key.to_le_bytes())?;
                        writer.write_all(&value.to_le_bytes())?;
                    }
                }
            }
        }
    } else {
        for &(start, size) in region {
            for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
                let size = proc.read_at(buf.as_mut_slice(), start + off)?;
                for (k, buf) in buf[..size].windows(PTRSIZE).enumerate() {
                    let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
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
                        writer.write_all(&key.to_le_bytes())?;
                        writer.write_all(&value.to_le_bytes())?;
                    }
                }
            }
        }
    }

    Ok(())
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
}
