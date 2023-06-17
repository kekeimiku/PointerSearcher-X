use std::{collections::BTreeMap, io, ops::Bound::Included};

pub struct Params<'a, W> {
    pub base: u64,
    pub depth: u64,
    pub range: (u64, u64),
    pub points: &'a [u64],
    pub target: u64,
    pub writer: &'a mut W,
}

#[inline(always)]
fn signed_diff(a: u64, b: u64) -> i16 {
    a.checked_sub(b).map(|a| a as i16).unwrap_or_else(|| -((b - a) as i16))
}

pub fn pointer_seacher<W: io::Write>(map: &BTreeMap<u64, Vec<u64>>, params: Params<W>) -> io::Result<()> {
    let depth = params.depth;
    walk_down_binary(
        map,
        params,
        1,
        (&mut Vec::with_capacity(depth as _), &mut Vec::with_capacity((depth * 2 + 9) as _)),
    )
}

fn walk_down_binary<W>(
    map: &BTreeMap<u64, Vec<u64>>,
    params: Params<W>,
    lv: u64,
    (tmp_v, tmp_s): (&mut Vec<i16>, &mut Vec<u8>),
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
        // // TODO ignore pointer chains with depth less than 3?
        // if tmp_v.len() > 3 {
        //     let m = iter.min_by_key(|&e| signed_diff(target, e)).unwrap_or(m);
        //     let off = signed_diff(target, m);
        //     tmp_v.push(off);
        //     tmp_s.extend((m - base).to_le_bytes());
        //     tmp_s.extend(tmp_v.iter().flat_map(|x| x.to_le_bytes()));
        //     tmp_s.push(101);
        //     tmp_s.resize(tmp_s.capacity(), 0);
        //     writer.write_all(tmp_s)?;
        //     tmp_s.clear();
        //     tmp_v.pop();
        // }

        let m = iter.min_by_key(|&e| signed_diff(target, e)).unwrap_or(m);
        let off = signed_diff(target, m);
        tmp_v.push(off);
        tmp_s.extend((m - base).to_le_bytes());
        tmp_s.extend(tmp_v.iter().flat_map(|x| x.to_le_bytes()));
        tmp_s.push(101);
        tmp_s.resize(tmp_s.capacity(), 0);
        writer.write_all(tmp_s)?;
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
                    Params { target, base, writer, range: (lr, ur), depth, points },
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

    let points = ptrs
        .range((Included(0x104B18000), Included(0x104B38000)))
        .map(|(k, _)| k)
        .copied()
        .collect::<Vec<_>>();

    let mut map: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (k, v) in ptrs {
        map.entry(v).or_default().push(k);
    }

    let mut out = vec![];

    pointer_seacher(
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

    for v in out.chunks(19) {
        let (start, path) = v.split_at(8);
        let start = u64::from_le_bytes(start.try_into().unwrap());
        println!(
            "base+{start:#x}{}",
            path.rsplit(|x| 101.eq(x))
                .nth(1)
                .unwrap()
                .chunks(2)
                .rev()
                .map(|x| format!("{}", i16::from_le_bytes(x.try_into().unwrap())))
                .collect::<Vec<_>>()
                .join("->")
        );
    }

    assert_eq!(
        out,
        [
            40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 16, 0, 0, 0, 0, 0, 101, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
            0, 0, 101, 0, 0, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 0, 0, 0, 0
        ]
    );
}
