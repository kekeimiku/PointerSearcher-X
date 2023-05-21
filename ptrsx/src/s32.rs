use std::{collections::BTreeMap, io, ops::Bound::Included};

pub struct Params<'a, W> {
    pub target: u32,
    pub base: u32,
    pub writer: &'a mut W,
    pub range: (u32, u32),
    pub max_lv: u32,
    pub start: &'a [u32],
}

#[inline(always)]
fn signed_diff(a: u32, b: u32) -> i16 {
    a.checked_sub(b).map(|a| a as i16).unwrap_or_else(|| -((b - a) as i16))
}

pub fn pointer_seacher<W: io::Write>(map: &BTreeMap<u32, Vec<u32>>, params: Params<W>) -> io::Result<()> {
    let depth = params.max_lv;
    walk_down(map, params, 1, (&mut Vec::with_capacity(depth as _), &mut Vec::with_capacity((depth * 2 + 9) as _)))
}

#[inline]
fn walk_down<W>(
    map: &BTreeMap<u32, Vec<u32>>,
    params: Params<W>,
    lv: u32,
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
