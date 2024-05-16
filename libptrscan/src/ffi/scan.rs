use core::ops::{ControlFlow, Range};
use std::io::{BufWriter, Error, Write};

use crate::{
    dump::PointerMap,
    scan::{try_pointer_chain_scan, Chain, Param},
};

pub struct UserParam {
    pub param: Param,
    pub node: Option<usize>,
    pub last: Option<isize>,
    pub max: Option<usize>,
}

macro_rules! try_scan {
    ($m:expr) => {
        match $m {
            ControlFlow::Continue(_) => Ok(()),
            ControlFlow::Break(b) => b,
        }
    };
}

pub fn pointer_chain_scan(
    map: &PointerMap,
    w: impl Write,
    param: UserParam,
    symbol: &str,
) -> Result<(), Error> {
    let mut buffer = BufWriter::with_capacity(0x100000, w);
    let PointerMap { points, map, modules } = map;

    let UserParam { param, node, last, max, .. } = param;
    match (node, last, max) {
        (None, None, None) => {
            let mut f = |chain: Chain| {
                let addr = chain.addr();
                let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                else {
                    return ControlFlow::Continue(());
                };
                match write!(buffer, "{name}+{:X}", addr - start)
                    .and(chain.data().try_for_each(|&o| {
                        if o >= 0 {
                            write!(buffer, "{symbol}{o:X}")
                        } else {
                            write!(buffer, "{symbol}-{:X}", o.abs())
                        }
                    }))
                    .and(writeln!(buffer))
                {
                    Ok(_) => ControlFlow::Continue(()),
                    Err(err) => ControlFlow::Break(Err(err)),
                }
            };
            try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
        }
        (None, None, Some(max)) => {
            let mut n = 0;
            let mut f = |chain: Chain| {
                if n >= max {
                    return ControlFlow::Break(Ok(()));
                }
                let addr = chain.addr();
                let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                else {
                    return ControlFlow::Continue(());
                };
                match write!(buffer, "{name}+{:X}", addr - start)
                    .and(chain.data().try_for_each(|&o| {
                        if o >= 0 {
                            write!(buffer, "{symbol}{o:X}")
                        } else {
                            write!(buffer, "{symbol}-{:X}", o.abs())
                        }
                    }))
                    .and(writeln!(buffer))
                {
                    Ok(_) => {
                        n += 1;
                        ControlFlow::Continue(())
                    }
                    Err(err) => ControlFlow::Break(Err(err)),
                }
            };
            try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
        }
        (None, Some(last), None) => {
            let mut f = |chain: Chain| {
                if chain.last().is_some_and(|o| last.eq(o)) {
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => ControlFlow::Continue(()),
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
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
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => {
                            n += 1;
                            ControlFlow::Continue(())
                        }
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
            };
            try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
        }
        (Some(node), None, None) => {
            let mut f = |chain: Chain| {
                if chain.len() >= node {
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => ControlFlow::Continue(()),
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
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
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => {
                            n += 1;
                            ControlFlow::Continue(())
                        }
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
            };
            try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
        }
        (Some(node), Some(last), None) => {
            let mut f = |chain: Chain| {
                if chain
                    .last()
                    .is_some_and(|o| chain.len() >= node && last.eq(o))
                {
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => ControlFlow::Continue(()),
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
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
                    let addr = chain.addr();
                    let Some((Range { start, .. }, name)) = modules.get_key_value_by_point(&addr)
                    else {
                        return ControlFlow::Continue(());
                    };
                    return match write!(buffer, "{name}+{:X}", addr - start)
                        .and(chain.data().try_for_each(|&o| {
                            if o >= 0 {
                                write!(buffer, "{symbol}{o:X}")
                            } else {
                                write!(buffer, "{symbol}-{:X}", o.abs())
                            }
                        }))
                        .and(writeln!(buffer))
                    {
                        Ok(_) => {
                            n += 1;
                            ControlFlow::Continue(())
                        }
                        Err(err) => ControlFlow::Break(Err(err)),
                    };
                }
                ControlFlow::Continue(())
            };
            try_scan!(try_pointer_chain_scan(map, points, param, &mut f))
        }
    }?;

    Ok(())
}
