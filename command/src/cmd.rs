use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::{BufWriter, Write},
    mem,
    path::{Path, PathBuf},
    thread,
};

use argh::{FromArgValue, FromArgs};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    ThreadPool, ThreadPoolBuilder,
};

use super::{error::Result, ptrscan::*, utils::Spinner};

#[derive(Debug)]
pub struct AddressList(pub Vec<usize>);

impl FromArgValue for AddressList {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let bset = value
            .split(['-', ',', ';'])
            .filter(|s| !s.is_empty())
            .map(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(Self(bset.into_iter().collect()))
    }
}

#[derive(Debug)]
pub struct Range(pub (usize, usize));

impl FromArgValue for Range {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let (lr, ur) = value
            .split_once(':')
            .ok_or(format!("parse command failed: {value}"))?;
        let lr = lr
            .trim_start_matches('-')
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let ur = ur
            .trim_start_matches('+')
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        Ok(Self((lr, ur)))
    }
}

#[derive(FromArgs, Debug)]
#[argh(description = "ptrscan")]
pub struct PointerSearch {
    #[argh(subcommand)]
    pub cmds: TopCommandEnum,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum TopCommandEnum {
    ListModules(ListModules),
    CreatePointerMap(CreatePointerMap),
    ScanPointerChain(ScanPointerChain),
    CmpPointerChain(CmpPointerChain),
    TestPointerChain(TestPointerChain),
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "list_modules",
    description = "get base address module"
)]
pub struct ListModules {
    #[argh(option, description = "process id")]
    pub pid: i32,

    #[argh(option, description = "output to file path")]
    pub output_file: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "create_pointer_map",
    description = "create pointer_map file"
)]
pub struct CreatePointerMap {
    #[argh(option, description = "process id")]
    pub pid: i32,

    #[argh(option, description = "module list file path")]
    pub use_modules_file: Option<PathBuf>,

    #[argh(option, description = "output to file path")]
    pub output_file: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum ScanCommandEnum {
    UsePointerMapFile(UsePointerMapFile),
    UseProcessID(UseProcessID),
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "use_pointer_map",
    description = "scan pointer chain, use pointer_map"
)]
pub struct UsePointerMapFile {
    #[argh(option, description = "pointer_map file path")]
    pub file: PathBuf,

    #[argh(option, description = "address list")]
    pub addr_list: AddressList,

    #[argh(option, description = "scan depth")]
    pub depth: usize,

    #[argh(option, description = "scan range")]
    pub range: Range,

    #[argh(option, description = "scan last range")]
    pub last_range: Option<Range>,

    #[argh(option, description = "scan node")]
    pub node: Option<usize>,

    #[argh(option, description = "max pointer chain list")]
    pub max_num: Option<usize>,

    #[argh(option, description = "last offset")]
    pub last_offset: Option<isize>,

    #[argh(switch, description = "filter pointer cycle ref")]
    pub filter_cycle_ref: bool,

    #[argh(option, description = "output to dir")]
    pub output_dir: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "use_process",
    description = "scan pointer chain, use process id"
)]
pub struct UseProcessID {
    #[argh(option, description = "process id")]
    pub pid: i32,

    #[argh(option, description = "base modules file")]
    pub use_modules_file: Option<PathBuf>,

    #[argh(option, description = "address list")]
    pub addr_list: AddressList,

    #[argh(option, description = "scan depth")]
    pub depth: usize,

    #[argh(option, description = "scan range")]
    pub range: Range,

    #[argh(option, description = "scan last range")]
    pub last_range: Option<Range>,

    #[argh(option, description = "scan node")]
    pub node: Option<usize>,

    #[argh(option, description = "max chain list num")]
    pub max_num: Option<usize>,

    #[argh(option, description = "last offset")]
    pub last_offset: Option<isize>,

    #[argh(switch, description = "filter pointer chain cycle ref")]
    pub filter_cycle_ref: bool,

    #[argh(option, description = "output to dir")]
    pub output_dir: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "scan_pointer_chain",
    description = "scan pointer chain"
)]
pub struct ScanPointerChain {
    #[argh(subcommand)]
    pub cmds: ScanCommandEnum,
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "cmp_pointer_chain",
    description = "compare pointer chain files"
)]
pub struct CmpPointerChain {
    #[argh(option, description = "pointer chain file1")]
    pub file1: PathBuf,

    #[argh(option, description = "pointer chain file2")]
    pub file2: PathBuf,

    #[argh(option, description = "output to file path")]
    pub output_file: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(
    subcommand,
    name = "test_pointer_chain",
    description = "test pointer chain"
)]
pub struct TestPointerChain {
    #[argh(option, description = "process id")]
    pub pid: i32,

    #[argh(option, description = "poiner chain")]
    pub chain: PointerChain,
}

#[cfg(target_os = "android")]
fn filter_modules(m: &&Module) -> bool {
    const N_ELFS: [&str; 3] = ["oat", "dex", "odex"];
    let path = Path::new(&m.pathname);
    path.starts_with("/data")
        && !path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| N_ELFS.contains(&s))
}

#[cfg(target_os = "macos")]
fn filter_modules(m: &&Module) -> bool {
    !Path::new(&m.pathname).starts_with("/usr/")
}

#[cfg(target_os = "linux")]
fn filter_modules(_m: &&Module) -> bool {
    true
}

#[cfg(target_os = "ios")]
fn filter_modules(m: &&Module) -> bool {
    let path = Path::new(&m.pathname);
    !(path.starts_with("/usr/")
        || path.starts_with("/System/Library/")
        || path.starts_with("/private/preboot/"))
        || path.starts_with("/Library/MobileSubstrate/")
}

fn modules_mask(modules: &[Module]) -> Vec<Module> {
    let mut result = Vec::with_capacity(modules.len());
    let mut counts: HashMap<&str, usize> = HashMap::new();

    for &Module { start, end, ref pathname } in modules.iter().filter(filter_modules) {
        let filename = Path::new(pathname)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(pathname);
        let count = counts.entry(filename).or_insert(0);
        let pathname = format!("{filename}[{count}]");
        *count += 1;
        result.push(Module { start, end, pathname })
    }

    result
}

impl ListModules {
    pub fn init(self) -> Result<()> {
        let Self { pid, output_file } = self;

        let mut ptrscan = PointerScan::new();
        ptrscan.attach_process(pid)?;

        let modules = ptrscan.list_modules()?;
        let mask_modules = modules_mask(&modules);

        let output_file =
            output_file.unwrap_or_else(|| PathBuf::new().join(format!("{pid}.modules.txt")));
        let file = File::options()
            .append(true)
            .create_new(true)
            .open(&output_file)?;
        let mut writer = BufWriter::new(file);

        mask_modules
            .iter()
            .try_for_each(|Module { start, end, pathname }| {
                writeln!(writer, "{start:x}-{end:x} {pathname}")
            })?;

        println!("output_file = {}", output_file.display());

        Ok(())
    }
}

impl CmpPointerChain {
    pub fn init(self) -> Result<()> {
        let Self { file1, file2, output_file } = self;

        let s1 = fs::read_to_string(&file1)?;
        let s2 = fs::read_to_string(&file2)?;
        let set1 = s1.lines().collect::<HashSet<_>>();
        let set2 = s2.lines().collect::<HashSet<_>>();

        let output_file = output_file.unwrap_or_else(|| {
            PathBuf::new().join(format!(
                "{}-{}-cmp.txt",
                file1
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| file1.file_name().and_then(|s| s.to_str()).unwrap()),
                file2
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| file2.file_name().and_then(|s| s.to_str()).unwrap()),
            ))
        });
        let file = File::options()
            .append(true)
            .create_new(true)
            .open(output_file)?;

        let mut writer = BufWriter::new(file);

        set1.intersection(&set2)
            .try_for_each(|s| writeln!(writer, "{s}"))?;

        Ok(())
    }
}

pub fn rayon_create_pool(num_threads: usize) -> Result<ThreadPool> {
    let num_cpus = thread::available_parallelism()?.get();
    let num = num_threads.min(num_cpus);
    let pool = ThreadPoolBuilder::new()
        .num_threads(num)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(pool)
}

impl UsePointerMapFile {
    pub fn init(self) -> Result<()> {
        let Self {
            file,
            addr_list,
            depth,
            range,
            last_range,
            node,
            max_num,
            last_offset,
            filter_cycle_ref,
            output_dir,
        } = self;

        let AddressList(addr_list) = addr_list;
        let Range(range) = range;
        let last_range = last_range.map(|r| r.0);

        let mut ptrscan = PointerScan::new();

        let mut spinner = Spinner::start("load_pointer_map ...");
        ptrscan.load_pointer_map_file(file)?;
        spinner.stop("load_pointer_map finished.");

        let mut spinner = Spinner::start("scan_pointer_chain ...");
        rayon_create_pool(addr_list.len())?.install(|| {
            addr_list.into_par_iter().try_for_each(|addr| {
                let param = Param {
                    addr,
                    depth,
                    srange: range,
                    lrange: last_range,
                    node,
                    last: last_offset,
                    max: max_num,
                    cycle: filter_cycle_ref,
                };
                let output_file = output_dir
                    .clone()
                    .unwrap_or_default()
                    .join(format!("{addr:x}.scandata"));
                ptrscan.scan_pointer_chain(param, output_file)
            })
        })?;
        spinner.stop(format!(
            "scan_pointer_chain finished. output_dir = {} *.scandata",
            output_dir.unwrap_or_default().display()
        ));

        Ok(())
    }
}

impl UseProcessID {
    pub fn init(self) -> Result<()> {
        let Self {
            pid,
            use_modules_file,
            addr_list,
            depth,
            range,
            last_range,
            node,
            max_num,
            last_offset,
            filter_cycle_ref,
            output_dir,
        } = self;

        let AddressList(addr_list) = addr_list;
        let Range(range) = range;
        let last_range = last_range.map(|r| r.0);

        let mut ptrscan = PointerScan::new();
        ptrscan.attach_process(pid)?;

        let modules = match use_modules_file {
            Some(filepath) => {
                let contents = fs::read_to_string(filepath)?;
                ModuleIter::new(&contents).collect()
            }
            None => {
                let modules = ptrscan.list_modules()?;
                modules_mask(&modules)
            }
        };

        let mut spinner = Spinner::start("create_pointer_map ...");
        ptrscan.create_pointer_map(modules)?;
        spinner.stop("create_pointer_map finished.");

        let mut spinner = Spinner::start("scan_pointer_chain ...");
        rayon_create_pool(addr_list.len())?.install(|| {
            addr_list.into_par_iter().try_for_each(|addr| {
                let param = Param {
                    addr,
                    depth,
                    srange: range,
                    lrange: last_range,
                    node,
                    last: last_offset,
                    max: max_num,
                    cycle: filter_cycle_ref,
                };
                let output_file = output_dir
                    .clone()
                    .unwrap_or_default()
                    .join(format!("{addr:x}.scandata"));
                ptrscan.scan_pointer_chain(param, output_file)
            })
        })?;

        spinner.stop(format!(
            "scan_pointer_chain finished. output_dir = {} *.scandata",
            output_dir.unwrap_or_default().display()
        ));

        Ok(())
    }
}

struct ModuleIter<'a>(core::str::Lines<'a>);

impl<'a> ModuleIter<'a> {
    fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = Module;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(2, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let pathname = split.next()?.trim().to_string();
        Some(Module { start, end, pathname })
    }
}

impl CreatePointerMap {
    pub fn init(self) -> Result<()> {
        let Self { pid, use_modules_file, output_file } = self;

        let mut ptrscan = PointerScan::new();
        ptrscan.attach_process(pid)?;

        let modules = match use_modules_file {
            Some(filepath) => {
                let contents = fs::read_to_string(filepath)?;
                ModuleIter::new(&contents).collect()
            }
            None => {
                let modules = ptrscan.list_modules()?;
                modules_mask(&modules)
            }
        };

        let output_file =
            output_file.unwrap_or_else(|| PathBuf::new().join(format!("{pid}.pointer_map.bin")));

        let mut spinner = Spinner::start("create_pointer_map ...");
        ptrscan.create_pointer_map_file(modules, &output_file)?;

        spinner.stop(format!(
            "create_pointer_map ok. path = {}",
            output_file.display()
        ));

        Ok(())
    }
}

#[derive(Debug)]
pub struct PointerChain(pub (String, usize, Vec<isize>));

pub fn parse_pointer_chain(value: &str) -> Option<(String, usize, Vec<isize>)> {
    let (module_name, b) = value.rsplit_once('+')?;
    let mut iter = b.split('.');
    let module_offset = iter
        .next()
        .and_then(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16).ok())?;
    let pointer_chain = iter
        .map(|s| isize::from_str_radix(s.trim_start_matches("0x"), 16))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some((module_name.to_string(), module_offset, pointer_chain))
}

impl FromArgValue for PointerChain {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let chain = parse_pointer_chain(value).ok_or("invalid pointer_chain")?;
        Ok(Self(chain))
    }
}

impl TestPointerChain {
    pub fn init(self) -> Result<()> {
        let Self { pid, chain } = self;
        let PointerChain((name, offset, chain)) = chain;

        let mut ptrscan = PointerScan::new();
        ptrscan.attach_process(pid)?;
        let modules = ptrscan.list_modules()?;
        let mask_modules = modules_mask(&modules);

        let mut address = mask_modules
            .iter()
            .find(|Module { pathname, .. }| name.eq(pathname))
            .and_then(|Module { start, .. }| start.checked_add(offset))
            .ok_or("invalid base address")?;

        println!("{name} + {offset:x} = {address:x}");

        let mut buf = [0_u8; mem::size_of::<usize>()];
        for offset in chain {
            ptrscan.read_memory_exact(address, &mut buf)?;
            address = usize::from_ne_bytes(buf)
                .checked_add_signed(offset)
                .ok_or("invalid offset address")?;
            println!("+ {offset:x} = {address:x}");
        }

        Ok(())
    }
}
