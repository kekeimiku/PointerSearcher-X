use std::{collections::BTreeMap, io, ops::Bound::Included};

pub struct Params<'a, W> {
    pub target: u64,
    pub base: u64,
    pub writer: &'a mut W,
    pub range: (u64, u64),
    pub max_lv: u64,
    pub start: &'a [u64],
}

#[inline(always)]
fn signed_diff(a: u64, b: u64) -> i16 {
    a.checked_sub(b).map(|a| a as i16).unwrap_or_else(|| -((b - a) as i16))
}

pub fn pointer_seacher<W: io::Write>(map: &BTreeMap<u64, Vec<u64>>, params: Params<W>) -> io::Result<()> {
    let depth = params.max_lv;
    walk_down(map, params, 1, (&mut Vec::with_capacity(depth as _), &mut Vec::with_capacity((depth * 2 + 9) as _)))
}

#[inline]
fn walk_down<W>(
    map: &BTreeMap<u64, Vec<u64>>,
    params: Params<W>,
    lv: u64,
    (tmp_v, tmp_s): (&mut Vec<i16>, &mut Vec<u8>),
) -> io::Result<()>
where
    W: io::Write,
{
    let Params { target, base, writer: out, range: (lr, ur), max_lv, start } = params;

    let min = target.saturating_sub(ur);
    let max = target.saturating_add(lr);

    let idx = start.binary_search(&min).unwrap_or_else(|x| x);

    let mut iter = start.iter().skip(idx).take_while(|&v| v <= &max).copied();

    if let Some(m) = iter.next() {
        let m = iter.min_by_key(|&e| signed_diff(target, e)).unwrap_or(m);
        let off = signed_diff(target, m);
        tmp_v.push(off);
        tmp_s.extend_from_slice(&(m - base).to_le_bytes());
        tmp_s.extend(tmp_v.iter().flat_map(|x| x.to_le_bytes()));
        tmp_s.push(101);
        tmp_s.resize(tmp_s.capacity(), 0);
        out.write_all(tmp_s)?;
        tmp_s.clear();
        tmp_v.pop();
    }

    if lv < max_lv {
        for (&k, vec) in map.range((Included(min), Included(max))) {
            let off = signed_diff(target, k);
            tmp_v.push(off);
            for &target in vec {
                walk_down(
                    map,
                    Params { target, base, writer: out, range: (lr, ur), max_lv, start },
                    lv + 1,
                    (tmp_v, tmp_s),
                )?;
            }
            tmp_v.pop();
        }
    }

    Ok(())
}

#[test]
fn test_path_find_helpers() {
    let ptrs = BTreeMap::from([
        (0x104B28008, 0x125F040A0),
        (0x104B28028, 0x125F04090),
        (0x104B281B0, 0x125F040E0),
        (0x125F04090, 0x125F04080),
    ]);

    let target = 0x125F04080;
    let range = (0, 16);
    let max_lv = 5;
    let size = max_lv * 2 + 9;

    let mut map: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (&k, &v) in &ptrs {
        map.entry(v).or_default().push(k);
    }

    let start = ptrs
        .range((Included(0x104B18000), Included(0x104B38000)))
        .map(|(k, _)| k)
        .copied()
        .collect::<Vec<_>>();

    let mut out = vec![];
    pointer_seacher(
        &map,
        Params {
            target,
            base: 0x104B18000,
            writer: &mut out,
            range,
            max_lv,
            start: &start,
        },
    )
    .unwrap();

    let out = out.chunks(size as _).flatten().copied().collect::<Vec<_>>();

    assert_eq!(
        out,
        [
            40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 16, 0, 0, 0, 0, 0, 101, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
            0, 0, 101, 0, 0, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 0, 0, 0, 0
        ]
    );
}
