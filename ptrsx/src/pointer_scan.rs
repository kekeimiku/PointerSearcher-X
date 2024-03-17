use core::{
    iter,
    ops::{Bound, ControlFlow},
};
use std::collections::BTreeMap;

use super::try_trait::{FromResidual, Try};

pub struct Param {
    pub depth: usize,
    pub addr: usize,
    pub range: (usize, usize),
}

// large amounts data
fn _try_chain_scan_1<F, R>(map: &BTreeMap<usize, Vec<usize>>, points: &[usize], param: Param, f: &mut F) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let mut data = Vec::with_capacity(param.depth);
    __try_chain_scan_1(map, points, param, f, &mut data, 0)
}

fn __try_chain_scan_1<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: Param,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let Param { depth, addr, range } = param;
    let min = addr.saturating_sub(range.1);
    let max = addr.saturating_add(range.0);

    let idx = points.binary_search(&min).unwrap_or_else(|x| x);

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let branch = f(Chain { addr, data });
        match Try::branch(branch) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let branch = __try_chain_scan_1(map, points, Param { depth, addr, range }, f, data, curr + 1);
                match Try::branch(branch) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    };

    Try::from_output(())
}

// small amount data
fn _try_chain_scan_2<F, R>(map: &BTreeMap<usize, Vec<usize>>, points: &[usize], param: Param, f: &mut F) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let mut data = Vec::with_capacity(param.depth);
    __try_chain_scan_2(map, points, param, f, &mut data, 0)
}

fn __try_chain_scan_2<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: Param,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let Param { depth, addr, range } = param;
    let min = addr.saturating_sub(range.1);
    let max = addr.saturating_add(range.0);

    let idx = points.iter().position(|x| min.le(x)).unwrap_or(points.len());

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let branch = f(Chain { addr, data });
        match Try::branch(branch) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let branch = __try_chain_scan_2(map, points, Param { depth, addr, range }, f, data, curr + 1);
                match Try::branch(branch) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    };
    Try::from_output(())
}

pub fn try_pointer_chain_scan<F, R>(map: &BTreeMap<usize, Vec<usize>>, points: &[usize], param: Param, f: &mut F) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let count = map.values().filter(|v| v.len() < 64).count();
    match (map.len() - count).checked_mul(256) {
        Some(n) if n < count => _try_chain_scan_2(map, points, param, f),
        _ => _try_chain_scan_1(map, points, param, f),
    }
}

pub struct Chain<'a> {
    addr: usize,
    data: &'a [(usize, isize)],
}

impl Chain<'_> {
    // 获取基址
    #[inline]
    pub const fn addr(&self) -> usize {
        self.addr
    }

    // 获取指针链数据
    #[inline]
    pub fn data(&self) -> impl Iterator<Item = &isize> {
        self.data.iter().rev().map(|(_, o)| o)
    }

    // 获取指针链长度
    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    // // 获取指针链第一个偏移
    // #[inline]
    // pub fn first(&self) -> Option<&isize> {
    //     self.data.last().map(|(_, o)| o)
    // }

    // 获取指针链最后一个偏移
    #[inline]
    pub fn last(&self) -> Option<&isize> {
        self.data.first().map(|(_, o)| o)
    }

    // 检查循环引用
    // Some 返回过滤后的指针链，None 表示不存在循环引用
    #[inline]
    pub fn ref_cycle(&self) -> Option<impl Iterator<Item = &isize>> {
        let (first, rest) = self.data.split_first()?;
        let n = rest.iter().position(|x| x.0 == first.0)?;
        Some(iter::once(first).chain(rest.iter().skip(n + 1)).rev().map(|(_, o)| o))
    }
}
