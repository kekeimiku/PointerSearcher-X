use std::{collections::BTreeMap, io, ops::Bound::Included};

use arrayvec::{ArrayString, ArrayVec};

pub struct Params<'a, W> {
    pub base: usize,
    pub depth: usize,
    pub range: (usize, usize),
    pub points: &'a [usize],
    pub target: usize,
    pub writer: &'a mut W,
}

pub fn pointer_search<W>(map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()>
where
    W: io::Write,
{
    walk_down_binary(
        map,
        params,
        1,
        (&mut ArrayVec::new_const(), &mut ArrayString::new_const(), &mut itoa::Buffer::new()),
    )
}

fn walk_down_binary<W>(
    map: &BTreeMap<usize, Vec<usize>>,
    params: Params<W>,
    lv: usize,
    (tmp_v, tmp_s, itoa): (&mut ArrayVec<isize, 32>, &mut ArrayString<0x400>, &mut itoa::Buffer),
) -> io::Result<()>
where
    W: io::Write,
{
    let Params { base, depth, range: (lr, ur), points, target, writer } = params;

    let min = target.saturating_sub(ur);
    let max = target.saturating_add(lr);

    let idx = points.binary_search(&min).unwrap_or_else(|x| x);

    let mut iter = points.iter().skip(idx).take_while(|&v| v <= &max).copied();

    if let Some(m) = iter.next() {
        let m = iter.min_by_key(|&e| signed_diff(target, e)).unwrap_or(m);
        let off = signed_diff(target, m);
        tmp_v.push(off);
        tmp_s.push_str(itoa.format(m - base));
        for &s in tmp_v.iter().rev() {
            tmp_s.push('@');
            tmp_s.push_str(itoa.format(s))
        }
        tmp_s.push('\n');
        writer.write_all(tmp_s.as_bytes())?;
        tmp_s.clear();
        tmp_v.pop();
    }

    if lv < depth {
        for (&k, vec) in map.range((Included(min), Included(max))) {
            let off = signed_diff(target, k);
            tmp_v.push(off);
            for &target in vec {
                walk_down_binary(
                    map,
                    Params { base, depth, range: (lr, ur), points, target, writer },
                    lv + 1,
                    (tmp_v, tmp_s, itoa),
                )?;
            }
            tmp_v.pop();
        }
    }

    Ok(())
}

#[inline(always)]
fn signed_diff(a: usize, b: usize) -> isize {
    a.checked_sub(b)
        .map(|a| a as isize)
        .unwrap_or_else(|| -((b - a) as isize))
}

#[test]
fn test_path_find_helpers() {
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

    let mut out = vec![];

    pointer_search(
        &map,
        Params {
            base: 0x104B18000,
            depth: 5,
            range: (0, 16),
            points: &points,
            target: 0x125F04080,
            writer: &mut out,
        },
    )
    .unwrap();

    println!("{}", String::from_utf8(out.clone()).unwrap());

    assert_eq!(
        out,
        [
            54, 53, 53, 55, 54, 64, 48, 64, 48, 64, 49, 54, 64, 49, 54, 64, 48, 10, 54, 53, 53, 55, 54, 64, 48, 64, 48,
            64, 49, 54, 64, 48, 10, 54, 53, 53, 55, 54, 64, 48, 64, 48, 64, 48, 10
        ]
    );
}
