#![allow(clippy::len_without_is_empty)]
#![cfg_attr(feature = "nightly", feature(try_trait_v2))]

extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};
use core::{
    iter,
    ops::{Bound, ControlFlow},
};

#[cfg(not(feature = "nightly"))]
pub mod private {

    use core::{convert::Infallible, ops::ControlFlow};

    pub trait Try: FromResidual {
        type Output;

        type Residual;

        fn from_output(output: Self::Output) -> Self;

        fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
    }

    pub trait FromResidual<R = <Self as Try>::Residual> {
        fn from_residual(residual: R) -> Self;
    }

    impl<B, C> Try for ControlFlow<B, C> {
        type Output = C;
        type Residual = ControlFlow<B, Infallible>;

        #[inline]
        fn from_output(output: Self::Output) -> Self {
            ControlFlow::Continue(output)
        }

        #[inline]
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                ControlFlow::Continue(c) => ControlFlow::Continue(c),
                ControlFlow::Break(b) => ControlFlow::Break(ControlFlow::Break(b)),
            }
        }
    }

    impl<B, C> FromResidual for ControlFlow<B, C> {
        #[inline]
        fn from_residual(residual: ControlFlow<B, Infallible>) -> Self {
            match residual {
                ControlFlow::Break(b) => ControlFlow::Break(b),
                _ => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    impl<T> Try for Option<T> {
        type Output = T;
        type Residual = Option<Infallible>;

        #[inline]
        fn from_output(output: Self::Output) -> Self {
            Some(output)
        }

        #[inline]
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Some(v) => ControlFlow::Continue(v),
                None => ControlFlow::Break(None),
            }
        }
    }

    impl<T> FromResidual for Option<T> {
        #[inline]
        fn from_residual(_: Option<Infallible>) -> Self {
            None
        }
    }

    impl<T, E> Try for Result<T, E> {
        type Output = T;
        type Residual = Result<Infallible, E>;

        #[inline]
        fn from_output(output: Self::Output) -> Self {
            Ok(output)
        }

        #[inline]
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Ok(v) => ControlFlow::Continue(v),
                Err(e) => ControlFlow::Break(Err(e)),
            }
        }
    }

    impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Result<T, F> {
        #[inline]
        #[track_caller]
        fn from_residual(residual: Result<Infallible, E>) -> Self {
            match residual {
                Err(e) => Err(From::from(e)),
                _ => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }
}

#[cfg(feature = "nightly")]
use core::ops::{FromResidual, Try};

#[cfg(not(feature = "nightly"))]
use private::{FromResidual, Try};

/// 第一级偏移级别
const CURR: usize = 0;

/// 顺序查找的阀值
const MIN: usize = 64;

/// 核心扫描参数
struct CoreParam {
    depth: usize,
    addr: usize,
    srange: (usize, usize),
}

/// 核心扫描参数扩展 允许指定第一个偏移范围
struct CoreParamExt {
    core: CoreParam,
    lrange: (usize, usize),
}

/// 扫描参数
pub struct Param {
    /// 深度
    pub depth: usize,
    /// 目标地址
    pub addr: usize,
    /// 核心偏移 向前偏移:向后偏移
    pub srange: (usize, usize),
    /// 第一级偏移可选 向前偏移:向后偏移
    /// 对于一级偏移极度不平衡的指针链 这个参数可以极大提高扫描速度
    pub lrange: Option<(usize, usize)>,
}

/// 指针链
pub struct Chain<'a> {
    addr: usize,
    data: &'a [(usize, isize)],
}

impl Chain<'_> {
    /// 获取基址
    #[inline]
    pub const fn addr(&self) -> usize {
        self.addr
    }

    /// 获取指针链偏移数据
    #[inline]
    pub fn data(&self) -> impl Iterator<Item = &isize> {
        self.data.iter().rev().map(|(_, o)| o)
    }

    /// 获取指针链内部原始数据 地址+偏移
    #[inline]
    pub fn raw_data(&self) -> impl Iterator<Item = &(usize, isize)> {
        self.data.iter()
    }

    /// 获取指针链长度
    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// 获取指针链第一个偏移
    /// 备注: 过滤这个参数并不能提高扫描速度，只影响结果数量
    #[inline]
    pub fn first(&self) -> Option<&isize> {
        self.data.last().map(|(_, o)| o)
    }

    /// 获取指针链最后一个偏移
    /// 备注: 过滤这个参数并不能提高扫描速度，只影响结果数量
    #[inline]
    pub fn last(&self) -> Option<&isize> {
        self.data.first().map(|(_, o)| o)
    }

    /// 检查循环引用 Some 返回过滤后的指针链，None 表示不存在循环引用
    #[inline]
    pub fn ref_cycle(&self) -> Option<impl Iterator<Item = &isize>> {
        let (first, rest) = self.data.split_first()?;
        let n = rest.iter().position(|x| x.0 == first.0)?;
        Some(
            iter::once(first)
                .chain(rest.iter().skip(n + 1))
                .rev()
                .map(|(_, o)| o),
        )
    }
}

/// 核心扫描实现 适用基址较多，每级指针偏移量比较平衡的情况
fn __try_chain_scan_core_1<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: CoreParam,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let CoreParam { depth, addr, srange } = param;

    let min = addr.saturating_sub(srange.1);
    let max = addr.saturating_add(srange.0);

    let idx = points.binary_search(&min).unwrap_or_else(|x| x);

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let r = f(Chain { addr, data });
        match Try::branch(r) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let r = __try_chain_scan_core_1(
                    map,
                    points,
                    CoreParam { depth, addr, srange },
                    f,
                    data,
                    curr + 1,
                );
                match Try::branch(r) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    }

    Try::from_output(())
}

/// 核心扫描的扩展实现 适用基址较多，只有第一级偏移较大的情况
fn __try_chain_scan_ext_1<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: CoreParamExt,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let CoreParamExt { core: CoreParam { depth, addr, srange }, lrange } = param;

    let (min, max) = if curr != CURR {
        (addr.saturating_sub(srange.1), addr.saturating_add(srange.0))
    } else {
        (addr.saturating_sub(lrange.1), addr.saturating_add(lrange.0))
    };

    let idx = points.binary_search(&min).unwrap_or_else(|x| x);

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let r = f(Chain { addr, data });
        match Try::branch(r) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let param = CoreParamExt { core: CoreParam { depth, addr, srange }, lrange };
                let r = __try_chain_scan_ext_1(map, points, param, f, data, curr + 1);
                match Try::branch(r) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    }

    Try::from_output(())
}

/// 核心扫描实现 适用基址较少，每级指针偏移量比较平衡的情况
fn __try_chain_scan_core_2<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: CoreParam,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let CoreParam { depth, addr, srange } = param;

    let min = addr.saturating_sub(srange.1);
    let max = addr.saturating_add(srange.0);

    let idx = points
        .iter()
        .position(|x| min.le(x))
        .unwrap_or(points.len());

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let r = f(Chain { addr, data });
        match Try::branch(r) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let r = __try_chain_scan_core_2(
                    map,
                    points,
                    CoreParam { depth, addr, srange },
                    f,
                    data,
                    curr + 1,
                );
                match Try::branch(r) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    }

    Try::from_output(())
}

/// 核心扫描的扩展实现 适用基址较少，只有第一级偏移较大的情况
fn __try_chain_scan_ext_2<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: CoreParamExt,
    f: &mut F,
    data: &mut Vec<(usize, isize)>,
    curr: usize,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let CoreParamExt { core: CoreParam { depth, addr, srange }, lrange } = param;

    let (min, max) = if curr != CURR {
        (addr.saturating_sub(srange.1), addr.saturating_add(srange.0))
    } else {
        (addr.saturating_sub(lrange.1), addr.saturating_add(lrange.0))
    };

    let idx = points
        .iter()
        .position(|x| min.le(x))
        .unwrap_or(points.len());

    if points
        .iter()
        .skip(idx)
        .take_while(|x| max.ge(x))
        .min_by_key(|x| (x.wrapping_sub(addr) as isize).abs())
        .is_some()
    {
        let r = f(Chain { addr, data });
        match Try::branch(r) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(b) => return FromResidual::from_residual(b),
        }
    }

    if curr < depth {
        for (&k, v) in map.range((Bound::Included(min), Bound::Included(max))) {
            data.push((k, addr.wrapping_sub(k) as isize));
            for &addr in v {
                let param = CoreParamExt { core: CoreParam { depth, addr, srange }, lrange };
                let r = __try_chain_scan_ext_2(map, points, param, f, data, curr + 1);
                match Try::branch(r) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
            data.pop();
        }
    }

    Try::from_output(())
}

/// 扫描指针链
///
/// `map` 指针图，key=指针 value=地址列表
///
/// `points` 作为基址的地址列表
///
/// `param` 控制扫描行为的参数 [`Param`]
///
/// `f` 每条扫描到指针链通过回调返回 根据 [`ControlFlow`]
/// 控制扫描 正常中断/异常中断/继续
pub fn try_pointer_chain_scan<F, R>(
    map: &BTreeMap<usize, Vec<usize>>,
    points: &[usize],
    param: Param,
    f: &mut F,
) -> R
where
    F: FnMut(Chain) -> R,
    R: Try<Output = ()>,
{
    let Param { depth, addr, srange, lrange } = param;
    let mut data = Vec::with_capacity(param.depth);
    match lrange {
        Some(last_range) => {
            let param =
                CoreParamExt { core: CoreParam { depth, addr, srange }, lrange: last_range };
            if points.len() > MIN {
                __try_chain_scan_ext_1(map, points, param, f, &mut data, CURR)
            } else {
                __try_chain_scan_ext_2(map, points, param, f, &mut data, CURR)
            }
        }
        None => {
            let param = CoreParam { depth, addr, srange };
            if points.len() > MIN {
                __try_chain_scan_core_1(map, points, param, f, &mut data, CURR)
            } else {
                __try_chain_scan_core_2(map, points, param, f, &mut data, CURR)
            }
        }
    }
}
