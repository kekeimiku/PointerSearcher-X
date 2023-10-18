use std::{
    fs::{File, OpenOptions},
    thread,
};

use ptrsx::{Param, PtrsxScanner};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    ThreadPool, ThreadPoolBuilder,
};

use super::{AddressList, Error, Range, Spinner, SubCommandScan};

impl SubCommandScan {
    pub fn init(self) -> Result<(), Error> {
        let Self {
            bin,
            info,
            list: AddressList(list),
            depth,
            range: Range(range),
            node,
            dir,
        } = self;

        if depth <= node {
            return Err(format!("depth must be greater than node. current depth({depth}), node({node}).").into());
        }

        let mut spinner = Spinner::start("start loading cache...");
        let mut ptrsx = PtrsxScanner::default();
        let info = File::open(info)?;
        ptrsx.load_modules_info(info)?;
        let bin = File::open(bin)?;
        ptrsx.load_pointer_map(bin)?;
        spinner.stop("cache load is finished.");

        let mut spinner = Spinner::start("start scanning pointer chain...");

        let dir = dir.unwrap_or_default();

        rayon_create_pool(list.len())?.install(|| {
            list.into_par_iter().try_for_each(|addr| {
                let path = dir.join(format!("{addr:x}")).with_extension("scandata");
                let file = OpenOptions::new().append(true).create_new(true).open(path)?;
                let param = Param { depth, addr, node, range };
                ptrsx.pointer_chain_scanner(param, file)
            })
        })?;

        spinner.stop("pointer chain scan is finished.");

        Ok(())
    }
}

pub fn rayon_create_pool(num_threads: usize) -> Result<ThreadPool, Error> {
    let num_cpus = thread::available_parallelism()?.get();
    let num = num_threads.min(num_cpus);
    let pool = ThreadPoolBuilder::new()
        .num_threads(num)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(pool)
}
