use core::ops::{ControlFlow, Range};
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    path::Path,
};

use crate::{
    dump::PointerMap,
    scan::{try_pointer_chain_scan, Chain, Param},
};

pub struct UserParam {
    pub param: Param,
    pub node: Option<usize>,
    pub last: Option<isize>,
    pub max: Option<usize>,
    pub cycle: bool,
}

macro_rules! try_scan {
    ($m:expr) => {
        match $m {
            ControlFlow::Continue(_) => Ok(()),
            ControlFlow::Break(b) => b,
        }
    };
}

macro_rules! output_pointer_chain {
    ($a:expr, $b:expr, $c:expr, $d:expr $(, $n:expr)?) => {{
        let addr = $a.addr();
        let Some((Range { start, .. }, name)) = $b.get_key_value_by_point(&addr) else {
            // 正常永远不会走到这里
            return ControlFlow::Continue(());
        };
        match write!($d, "{name}+{:X}", addr - start)
            .and($c.try_for_each(|&o| {
                if o >= 0 {
                    write!($d, ".{o:X}")
                } else {
                    write!($d, ".-{:X}", o.abs())
                }
            }))
            .and(writeln!($d))
        {
            Ok(_) => {
                $( $n += 1; )?
                ControlFlow::Continue(())
            },
            Err(err) => ControlFlow::Break(Err(err)),
        }
    }};
}

pub fn pointer_chain_scan(
    map: &PointerMap,
    path: impl AsRef<Path>,
    param: UserParam,
) -> Result<(), Error> {
    let file = File::options().append(true).create_new(true).open(path)?;
    let mut buffer = BufWriter::with_capacity(0x100000, file);
    let PointerMap { points, map, modules } = map;

    let UserParam { param, node, last, max, cycle } = param;
    if cycle {
        match (node, last, max) {
            (None, None, None) => {
                let mut f = |chain: Chain| match chain.ref_cycle() {
                    Some(mut iter) => output_pointer_chain!(chain, modules, iter, buffer),
                    None => output_pointer_chain!(chain, modules, chain.data(), buffer),
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, None, Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    match chain.ref_cycle() {
                        Some(mut iter) => output_pointer_chain!(chain, modules, iter, buffer, n),
                        None => output_pointer_chain!(chain, modules, chain.data(), buffer, n),
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, Some(last), None) => {
                let mut f = |chain: Chain| {
                    if chain.last().is_some_and(|o| last.eq(o)) {
                        match chain.ref_cycle() {
                            Some(mut iter) => output_pointer_chain!(chain, modules, iter, buffer),
                            None => output_pointer_chain!(chain, modules, chain.data(), buffer),
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, Some(last), Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain.last().is_some_and(|o| last.eq(o)) {
                        match chain.ref_cycle() {
                            Some(mut iter) => {
                                output_pointer_chain!(chain, modules, iter, buffer, n)
                            }
                            None => {
                                output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                            }
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), None, None) => {
                let mut f = |chain: Chain| {
                    if chain.len() >= node {
                        match chain.ref_cycle() {
                            Some(mut iter) => output_pointer_chain!(chain, modules, iter, buffer),
                            None => output_pointer_chain!(chain, modules, chain.data(), buffer),
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), None, Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain.len() >= node {
                        match chain.ref_cycle() {
                            Some(mut iter) => {
                                output_pointer_chain!(chain, modules, iter, buffer, n)
                            }
                            None => {
                                output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                            }
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), Some(last), None) => {
                let mut f = |chain: Chain| {
                    if chain
                        .last()
                        .is_some_and(|o| chain.len() >= node && last.eq(o))
                    {
                        match chain.ref_cycle() {
                            Some(mut iter) => output_pointer_chain!(chain, modules, iter, buffer),
                            None => output_pointer_chain!(chain, modules, chain.data(), buffer),
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), Some(last), Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain
                        .last()
                        .is_some_and(|o| chain.len() >= node && last.eq(o))
                    {
                        match chain.ref_cycle() {
                            Some(mut iter) => {
                                output_pointer_chain!(chain, modules, iter, buffer, n)
                            }
                            None => {
                                output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                            }
                        }
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
        }
    } else {
        match (node, last, max) {
            (None, None, None) => {
                let mut f =
                    |chain: Chain| output_pointer_chain!(chain, modules, chain.data(), buffer);
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, None, Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, Some(last), None) => {
                let mut f = |chain: Chain| {
                    if chain.last().is_some_and(|o| last.eq(o)) {
                        output_pointer_chain!(chain, modules, chain.data(), buffer)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (None, Some(last), Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain.last().is_some_and(|o| last.eq(o)) {
                        output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), None, None) => {
                let mut f = |chain: Chain| {
                    if chain.len() >= node {
                        output_pointer_chain!(chain, modules, chain.data(), buffer)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), None, Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain.len() >= node {
                        output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), Some(last), None) => {
                let mut f = |chain: Chain| {
                    if chain
                        .last()
                        .is_some_and(|o| chain.len() >= node && last.eq(o))
                    {
                        output_pointer_chain!(chain, modules, chain.data(), buffer)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
            (Some(node), Some(last), Some(max)) => {
                let mut n = 0;
                let mut f = |chain: Chain| {
                    if n >= max {
                        return ControlFlow::Break(Ok(()));
                    }
                    if chain
                        .last()
                        .is_some_and(|o| chain.len() >= node && last.eq(o))
                    {
                        output_pointer_chain!(chain, modules, chain.data(), buffer, n)
                    } else {
                        ControlFlow::Continue(())
                    }
                };
                try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
            }
        }
    }
}
