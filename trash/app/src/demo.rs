use core::ops::Range;
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use crate::{
    cache::{read_cache, write_cache, Offset, BUFFERSIZE},
    error::Result,
    proc::{MapsExt, MapsIter, Mem, MemExt},
    scan::{dump_mem, rescan, scan, ScanMode},
};

pub struct TmpPath {
    pub mem_tmp: PathBuf,
    pub addr_tmp: PathBuf,
}

pub struct Region<A, B> {
    pub map: A,
    pub mem: B,
    pub tmp: TmpPath,
    pub index: Vec<Range<usize>>,
    pub num: usize,
}

impl<A, B> Region<A, B>
where
    A: MapsExt,
    B: MemExt,
{
    pub fn new(map: A, mem: B) -> Self {
        let mem_tmp = PathBuf::from(format!("./LINCE_CACHE/{}", map.start()));
        let addr_tmp = PathBuf::from(format!("./LINCE_CACHE/{}", map.start()));
        let tmp = TmpPath { mem_tmp, addr_tmp };
        Self { map, mem, tmp, index: vec![], num: 0 }
    }

    pub fn scan(&mut self, value: &[u8]) -> Result<()> {
        dump_mem(&self.map, &self.mem, self.tmp.mem_tmp.as_path())?;
        let memory_cache = OpenOptions::new().read(true).open(self.tmp.mem_tmp.as_path())?;
        let result = scan(&self.map, memory_cache, value, ScanMode::Fast);
        let file = File::options().append(true).open(&self.tmp.addr_tmp)?;
        (self.index, self.num) = write_cache(result, file)?;

        Ok(())
    }

    // todo
    pub fn rescan(&mut self, value: &[u8]) -> Result<()> {
        dump_mem(&self.map, &self.mem, &self.tmp.mem_tmp)?;

        let _old_address_tmp_path = &self.tmp.addr_tmp;
        let _old_memory_tmp_path = &self.tmp.mem_tmp;

        let address_cache = File::open(&self.tmp.addr_tmp)?;
        let memory_cache = File::open(&self.tmp.mem_tmp)?;
        let memory_cache = Mem(memory_cache);

        let addr = read_cache::<Offset, File>(address_cache, &self.index);
        let result = rescan(&self.map, &memory_cache, value, addr);

        let new_address_tmp = File::options()
            .append(true)
            .create(true)
            .open(&self.tmp.addr_tmp)?;
        (self.index, self.num) = write_cache(result, new_address_tmp)?;

        Ok(())
    }

    pub fn undo(&mut self) {}

    pub fn get_value(&self) -> Result<()> {
        println!("总数: {}", self.num);
        let file = File::open(&self.tmp.addr_tmp)?;
        let mut num = 0;

        //todo 打印前10个地址
        for data in read_cache::<Offset, File>(file, &self.index) {
            let data = data?;
            for offset in data {
                println!("address: 0x{:x}", self.map.start() + num + (offset as usize));
            }
            num += BUFFERSIZE;
        }

        Ok(())
    }
}

pub fn prompt(name: &str) -> std::io::Result<Vec<String>> {
    let mut line = String::new();
    print!("{}", name);
    std::io::Write::flush(&mut std::io::stdout())?;
    std::io::stdin().read_line(&mut line)?;
    Ok(line
        .replace('\n', "")
        .split_whitespace()
        .map(String::from)
        .collect())
}

pub fn start() -> Result<()> {
    let pid = std::env::args().nth(1).unwrap().parse::<i32>()?;
    let contents = std::fs::read_to_string(format!("/proc/{pid}/maps"))?;
    let maps = MapsIter::new(&contents)
        .find(|m| m.pathname() == "[heap]")
        .unwrap();
    let mem = Mem::open_process(pid)?;

    let mut region = Region::new(maps, mem);

    loop {
        let prompt = prompt("> ").unwrap();
        let input = prompt.iter().map(String::as_str).collect::<Vec<&str>>();
        if input.is_empty() {
            println!("参数为空");
        } else {
            match input[0] {
                "type" => {}
                "find" => {
                    let value = input[1].parse::<i32>()?.to_ne_bytes();
                    region.scan(&value)?;
                }
                "get" => {
                    region.get_value()?;
                }
                "refind" => {
                    let value = input[1].parse::<i32>()?.to_ne_bytes();
                    region.rescan(&value)?;
                }
                "inject" => {}
                "write" => {}
                "read" => {}
                "ptr" => {}
                _ => {}
            }
        }
    }
}
