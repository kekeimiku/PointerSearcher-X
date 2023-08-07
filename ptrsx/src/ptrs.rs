use std::{collections::BTreeMap, io, ops::Bound::Included};

use arrayvec::{ArrayString, ArrayVec};

pub struct Params<'a, W> {
    pub base: usize,
    pub depth: usize,
    pub target: usize,
    pub node: usize,
    pub range: (usize, usize),
    pub points: &'a [usize],
    pub writer: &'a mut W,
}

type Tmp<'a> = (&'a mut ArrayVec<isize, 32>, &'a mut ArrayString<0x400>, &'a mut itoa::Buffer);

pub fn pointer_chain_scanner<W>(map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()>
where
    W: io::Write,
{
    scanner(map, params, 1, (&mut ArrayVec::new_const(), &mut ArrayString::new_const(), &mut itoa::Buffer::new()))
}

fn scanner<W>(map: &BTreeMap<usize, Vec<usize>>, params: Params<W>, lv: usize, tmp: Tmp) -> io::Result<()>
where
    W: io::Write,
{
    let Params { base, depth, target, node, range: (lr, ur), points, writer } = params;
    let (avec, astr, itoa) = tmp;

    let min = target.saturating_sub(ur);
    let max = target.saturating_add(lr);

    let idx = points.binary_search(&min).unwrap_or_else(|x| x);

    if points
        .iter()
        .skip(idx)
        .copied()
        .take_while(|&x| x <= max)
        .min_by_key(|&x| (target.wrapping_sub(x) as isize).abs())
        .map_or(false, |_| avec.len() >= node)
    {
        astr.push_str(itoa.format(target - base));
        avec.iter().rev().for_each(|&off| {
            astr.push('@');
            astr.push_str(itoa.format(off))
        });
        astr.push('\n');
        writer.write_all(astr.as_bytes())?;
        astr.clear();
    }

    if lv < depth {
        for (&k, vec) in map.range((Included(min), Included(max))) {
            avec.push(target.wrapping_sub(k) as isize);
            for &target in vec {
                scanner(
                    map,
                    Params { base, depth, target, node, range: (lr, ur), points, writer },
                    lv + 1,
                    (avec, astr, itoa),
                )?;
            }
            avec.pop();
        }
    }

    Ok(())
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_pointer_chain_scanner() {
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

    pointer_chain_scanner(
        &map,
        Params {
            base: 0x104B18000,
            depth: 5,
            target: 0x125F04080,
            node: 3,
            range: (0, 16),
            points: &points,
            writer: &mut out,
        },
    )
    .unwrap();

    println!("{}", String::from_utf8(out.clone()).unwrap());

    assert_eq!(
        out,
        [
            54, 53, 53, 55, 54, 64, 48, 64, 49, 54, 64, 49, 54, 64, 48, 10, 54, 53, 53, 55, 54, 64, 48, 64, 49, 54, 64,
            48, 10
        ]
    );
}
