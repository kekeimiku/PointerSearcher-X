use std::{cmp::Ordering, collections::BTreeMap, io, ops::Bound::Included};

use arrayvec::ArrayVec;

pub(super) struct Params<'a, W> {
    pub base: usize,
    pub depth: usize,
    pub target: usize,
    pub node: usize,
    pub offset: (usize, usize),
    pub points: &'a [usize],
    pub writer: &'a mut W,
}

// [usize] no dups optimized binary_search
#[inline(always)]
unsafe fn binary_search_by<'a, T, F>(slice: &'a [T], mut f: F) -> Result<usize, usize>
where
    F: FnMut(&'a T) -> Ordering,
{
    let mut size = slice.len();
    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        let cmp = f(slice.get_unchecked(mid));
        base = if cmp == Ordering::Greater { base } else { mid };
        size -= half;
    }
    let cmp: Ordering = f(slice.get_unchecked(base));
    if cmp == Ordering::Equal {
        Ok(base)
    } else {
        Err(base + (cmp == Ordering::Less) as usize)
    }
}

type Tmp<'a> = (&'a mut ArrayVec<isize, 32>, &'a mut itoa::Buffer);

pub fn pointer_chain_scanner<W>(map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()>
where
    W: io::Write,
{
    unsafe { scanner(map, params, 1, (&mut ArrayVec::new_const(), &mut itoa::Buffer::new())) }
}

#[inline(always)]
unsafe fn scanner<W>(map: &BTreeMap<usize, Vec<usize>>, params: Params<W>, lv: usize, tmp: Tmp) -> io::Result<()>
where
    W: io::Write,
{
    let Params { base, depth, target, node, offset: (lr, ur), points, writer } = params;
    let (avec, itoa) = tmp;

    let min = target.saturating_sub(ur);
    let max = target.saturating_add(lr);

    let idx = binary_search_by(points, |p| p.cmp(&min)).unwrap_or_else(|x| x);

    if points
        .iter()
        .skip(idx)
        .copied()
        .take_while(|&x| x <= max)
        .min_by_key(|&x| (target.wrapping_sub(x) as isize).abs())
        .is_some_and(|_| avec.len() >= node)
    {
        writer.write_all(itoa.format(target - base).as_bytes())?;
        for &off in avec.iter().rev() {
            writer.write_all(b"@")?;
            writer.write_all(itoa.format(off).as_bytes())?;
        }
        writer.write_all(b"\n")?;
    }

    if lv <= depth {
        for (&k, vec) in map.range((Included(min), Included(max))) {
            avec.push_unchecked(target.wrapping_sub(k) as isize);
            for &target in vec {
                scanner(
                    map,
                    Params { base, depth, target, node, offset: (lr, ur), points, writer },
                    lv + 1,
                    (avec, itoa),
                )?;
            }
            avec.pop();
        }
    }

    Ok(())
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_pointer_chain_scanner_s1() {
    let ptrs = BTreeMap::from([
        (0x104B28008, 0x125F040A0),
        (0x104B28028, 0x125F04090),
        (0x104B281B0, 0x125F040E0),
        (0x125F04090, 0x125F04080),
    ]);

    let points = ptrs
        .range((Included(0x104B18000), Included(0x104B38000)))
        .map(|(k, _)| k)
        .copied()
        .collect::<Vec<_>>();

    let mut map: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (k, v) in ptrs {
        map.entry(v).or_default().push(k);
    }

    let mut out = Vec::with_capacity(10);

    pointer_chain_scanner(
        &map,
        Params {
            base: 0x104B18000,
            depth: 4,
            target: 0x125F04080,
            node: 3,
            offset: (0, 16),
            points: &points,
            writer: &mut out,
        },
    )
    .unwrap();

    assert_eq!(String::from_utf8(out).unwrap(), "65576@0@16@16@0\n65576@0@16@0\n");
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_pointer_chain_scanner_s2() {
    let ptrs = BTreeMap::from([
        (0x104B28008, 0x125F040A0),
        (0x104B28028, 0x125F04090),
        (0x104B281B0, 0x125F040E0),
        (0x125F04090, 0x125F04080),
    ]);

    let mut map: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (k, v) in ptrs {
        map.entry(v).or_default().push(k);
    }

    let mut out = Vec::with_capacity(10);

    pointer_chain_scanner(
        &map,
        Params {
            base: 0,
            depth: 4,
            target: 0x125F04080,
            node: 3,
            offset: (0, 16),
            points: &[0x125F04090],
            writer: &mut out,
        },
    )
    .unwrap();

    assert_eq!(String::from_utf8(out).unwrap(), "4931469456@16@16@0\n4931469456@16@16@16@0\n");
}
