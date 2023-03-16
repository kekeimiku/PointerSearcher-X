use std::fmt::Display;

use crate::vmmap::{VirtualMemoryRead, VirtualQuery};

use super::error::{Error, Result};

type Address = usize;

#[cfg(target_os = "linux")]
const CHUNK: usize = 0x100000;

#[cfg(target_os = "macos")]
const CHUNK: usize = 0x4000;

pub struct Map {
    start: Address,
    pub size: Address,
    perm: Perm,
    pathname: String,
}

pub struct Perm((bool, bool, bool));

impl Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.0 .0 { "r" } else { "-" },
            if self.0 .1 { "w" } else { "-" },
            if self.0 .2 { "x" } else { "-" }
        )
    }
}

pub struct ValueScanner {
    pub maps: Vec<Map>,
    pub out: Vec<Vec<Vec<u32>>>,
    pub status: bool,
}

impl ValueScanner {
    pub fn init<I, V>(region: I) -> Self
    where
        V: VirtualQuery,
        I: Iterator<Item = V>,
    {
        Self {
            maps: region
                .map(|m| Map {
                    start: m.start(),
                    size: m.size(),
                    perm: Perm((m.is_read(), m.is_write(), m.is_exec())),
                    pathname: if !m.path().is_empty() {
                        m.path().to_owned()
                    } else if m.is_stack() {
                        String::from("stack")
                    } else if m.is_heap() {
                        String::from("heap")
                    } else {
                        String::from("misc")
                    },
                })
                .collect(),
            out: vec![],
            status: false,
        }
    }

    pub fn scan<P>(&mut self, proc: &P, data: &[u8]) -> Result<()>
    where
        P: VirtualMemoryRead + Clone + Sync,
    {
        let mut tmp_buf = vec![0; CHUNK];
        if !self.status {
            self.fscan(proc, data, tmp_buf.as_mut_slice())?;
            self.status = true;
        } else {
            self.rescan(proc, data, tmp_buf.as_mut_slice())?;
        }

        Ok(())
    }

    pub fn fscan<P>(&mut self, proc: &P, data: &[u8], tmp_buf: &mut [u8]) -> Result<()>
    where
        P: VirtualMemoryRead + Clone + Sync,
    {
        let mut tmp_v1 = vec![];
        let mut tmp_v2 = vec![];

        for Map { start, size, perm: _, pathname: _ } in &self.maps {
            for off in (0..*size).step_by(CHUNK) {
                let size = proc.read_at(start + off, tmp_buf).map_err(Error::Vmmap)?;
                tmp_v1.extend(
                    tmp_buf[..size]
                        .windows(data.len())
                        .enumerate()
                        .filter_map(|(k, v)| if v == data { Some(k as u32) } else { None }),
                );
                tmp_v2.push(tmp_v1.clone());
                tmp_v1.clear();
            }
            self.out.push(tmp_v2.clone());
            tmp_v2.clear();
        }

        Ok(())
    }

    pub fn rescan<P>(&mut self, proc: &P, data: &[u8], tmp_buf: &mut [u8]) -> Result<()>
    where
        P: VirtualMemoryRead + Clone + Sync,
    {
        for (Map { start, size, perm: _, pathname: _ }, vec) in self.maps.iter().zip(self.out.iter_mut()) {
            for (off, vec1) in (0..*size).step_by(CHUNK).zip(vec.iter_mut()) {
                if !vec1.is_empty() {
                    let size = proc.read_at(start + off, tmp_buf).map_err(Error::Vmmap)?;
                    let buf = &tmp_buf[..size];
                    vec1.retain(|&a| &buf[a as usize..data.len() + a as usize] == data);
                }
            }
        }

        Ok(())
    }

    pub fn print(&self) -> Result<()> {
        println!(
            "results: {}",
            self.out
                .iter()
                .map(|m| m.iter().map(|a| a.len()).sum::<usize>())
                .sum::<usize>()
        );

        let mut num = 0;
        'outer: for (Map { start, size, perm, pathname }, vec) in self.maps.iter().zip(self.out.iter()) {
            for (off, vec1) in (0..*size).step_by(CHUNK).zip(vec.iter()) {
                if !vec1.is_empty() {
                    for &k in vec1 {
                        println!("{:#x} {} {}", start + off + k as usize, perm, pathname);
                        num += 1;
                        if num == 10 {
                            break 'outer;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
