use core::{cmp::Ordering, mem, ops::ControlFlow};

use vmmap::{VirtualMemoryRead, VirtualQuery};

use super::try_trait::{FromResidual, Try};

struct ChunkIter {
    max: usize,
    size: usize,
    pos: usize,
}

impl ChunkIter {
    #[inline]
    fn new(max: usize, size: usize) -> Self {
        Self { max, size, pos: 0 }
    }
}

impl Iterator for ChunkIter {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.max {
            None
        } else {
            let curr = self.pos;
            self.pos = (self.pos + self.size).min(self.max);
            Some((curr, self.pos - curr))
        }
    }
}

#[inline]
fn is_pointer<V: VirtualQuery>(addr: &usize, vqs: &[V]) -> bool {
    vqs.binary_search_by(|vq| {
        let (start, size) = (vq.start(), vq.size());
        match (start..start + size).contains(addr) {
            true => Ordering::Equal,
            false => start.cmp(addr),
        }
    })
    .is_ok()
}

// memory align
fn _try_pointer_map1<P, V, F, R>(proc: &P, vqs: &[V], f: &mut F) -> R
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize) -> R,
    R: Try<Output = ()>,
{
    let mut buf = vec![0; 0x100000];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, 0x100000) {
            if proc.read_exact_at(&mut buf[..size], start + off).is_err() {
                break;
            }
            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .step_by(mem::size_of::<usize>())
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                let branch = f(start + off + k, v);
                match Try::branch(branch) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
        }
    }
    Try::from_output(())
}

// memory not align
fn _try_pointer_map2<P, V, F, R>(proc: &P, vqs: &[V], f: &mut F) -> R
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize) -> R,
    R: Try<Output = ()>,
{
    let mut buf = vec![0; 0x100000];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, 0x100000) {
            if proc.read_exact_at(&mut buf[..size], start + off).is_err() {
                break;
            }
            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                let branch = f(start + off + k, v);
                match Try::branch(branch) {
                    ControlFlow::Continue(c) => c,
                    ControlFlow::Break(b) => return FromResidual::from_residual(b),
                }
            }
        }
    }
    Try::from_output(())
}

// memory align
fn _pointer_map1<P, V, F>(proc: &P, vqs: &[V], f: &mut F)
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize),
{
    let mut buf = vec![0; 0x100000];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, 0x100000) {
            if proc.read_exact_at(&mut buf[..size], start + off).is_err() {
                break;
            }
            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .step_by(mem::size_of::<usize>())
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                f(start + off + k, v)
            }
        }
    }
}

// memory not align
fn _pointer_map2<P, V, F>(proc: &P, vqs: &[V], f: &mut F)
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize),
{
    let mut buf = vec![0; 0x100000];
    for vq in vqs {
        let (start, size) = (vq.start(), vq.size());
        for (off, size) in ChunkIter::new(size, 0x100000) {
            if proc.read_exact_at(&mut buf[..size], start + off).is_err() {
                break;
            }
            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(v, vqs))
            {
                f(start + off + k, v)
            }
        }
    }
}

// TODO: maybe make public
#[allow(dead_code)]
fn create_pointer_map<P, V, F>(proc: &P, vqs: &[V], align: bool, f: &mut F)
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize),
{
    match align {
        true => _pointer_map1(proc, vqs, f),
        false => _pointer_map2(proc, vqs, f),
    }
}

pub fn try_create_pointer_map<P, V, F, R>(proc: &P, vqs: &[V], align: bool, f: &mut F) -> R
where
    P: VirtualMemoryRead,
    V: VirtualQuery,
    F: FnMut(usize, usize) -> R,
    R: Try<Output = ()>,
{
    match align {
        true => _try_pointer_map1(proc, vqs, f),
        false => _try_pointer_map2(proc, vqs, f),
    }
}
