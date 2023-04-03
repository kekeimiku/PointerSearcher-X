use std::{cmp::Ordering, collections::BTreeMap, io, ops::Bound::Included};

use vmmap::VirtualMemoryRead;

use crate::consts::{Address, CHUNK_SIZE, POINTER_SIZE};

pub type PointerMap = BTreeMap<Address, Address>;

pub type ReversePointerMap = BTreeMap<Address, Vec<Address>>;

pub fn create_pointer_map<P>(proc: &P, region: &[(Address, Address)], out: &mut PointerMap)
where
    P: VirtualMemoryRead,
{
    let mut buf = vec![0; CHUNK_SIZE];
    let mut arr = [0; POINTER_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(CHUNK_SIZE) {
            let Ok (size) = proc.read_at(start + off, buf.as_mut_slice()) else {
                println!("skip {start:#x}-{:#x} read_err",start+size);
                break;
            };
            for (o, buf) in buf[..size].windows(POINTER_SIZE).enumerate() {
                let addr = start + off + o;
                arr[0..POINTER_SIZE].copy_from_slice(buf);
                let out_addr = Address::from_le_bytes(arr);
                if region
                    .binary_search_by(|&(a, s)| {
                        if out_addr >= a && out_addr < a + s {
                            Ordering::Equal
                        } else {
                            a.cmp(&out_addr)
                        }
                    })
                    .is_ok()
                {
                    out.insert(addr, out_addr);
                }
            }
        }
    }
}

struct WalkParams<'a, W> {
    target: Address,
    out: &'a mut W,
    range: (usize, usize),
    lv: usize,
    max_lv: usize,
    startpoints: &'a [Address],
}

#[inline(always)]
pub fn signed_diff(a: Address, b: Address) -> i16 {
    a.checked_sub(b).map(|a| a as i16).unwrap_or_else(|| -((b - a) as i16))
}

fn walk_down<W>(
    map: &ReversePointerMap,
    params: WalkParams<W>,
    (tmp_v, tmp_s): (&mut Vec<i16>, &mut Vec<u8>),
) -> Result<(), io::Error>
where
    W: io::Write,
{
    let WalkParams { target, out, range: (lr, ur), lv, max_lv, startpoints } = params;

    let min = target.saturating_sub(ur);
    let max = target.saturating_add(lr);

    let idx = startpoints.binary_search(&min).unwrap_or_else(|x| x);

    let mut iter = startpoints.iter().skip(idx).take_while(|&v| v <= &max).copied();

    if let Some(m) = iter.next() {
        let m = iter.min_by_key(|&e| signed_diff(target, e)).unwrap_or(m);
        let off = signed_diff(target, m);
        tmp_v.push(off);
        tmp_s.extend(m.to_le_bytes());
        let path = unsafe { core::slice::from_raw_parts(tmp_v.as_ptr() as *const u8, tmp_v.len() * 2) };
        tmp_s.extend(path);
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
                    WalkParams { target, out, range: (lr, ur), lv: lv + 1, max_lv, startpoints },
                    (tmp_v, tmp_s),
                )?;
            }
            tmp_v.pop();
        }
    }

    Ok(())
}

pub fn convert_rev_map(map: PointerMap) -> ReversePointerMap {
    let mut rev_map = ReversePointerMap::new();
    for (k, v) in map {
        rev_map.entry(v).or_default().push(k);
    }
    rev_map
}

pub fn path_find_helpers<W>(
    rev_map: ReversePointerMap,
    target: Address,
    out: &mut W,
    range: (usize, usize),
    max_lv: usize,
    startpoints: &[Address],
) -> Result<(), io::Error>
where
    W: io::Write,
{
    let params = WalkParams { target, out, range, lv: 1, max_lv, startpoints };
    let size = max_lv * 2 + 9;
    walk_down(&rev_map, params, (&mut Vec::with_capacity(max_lv), &mut Vec::with_capacity(size)))
}

#[test]
fn test_path_find_helpers() {
    let map = PointerMap::from([
        (0x104B28008, 0x125F040A0),
        (0x104B28028, 0x125F04090),
        (0x104B281B0, 0x125F040E0),
        (0x125F04090, 0x125F04080),
    ]);

    let startpoints = map
        .keys()
        .copied()
        .filter(|a| (0x104B18000..0x104B38000).contains(a))
        .collect::<Vec<_>>();

    let mut rev_map = ReversePointerMap::new();
    for (k, v) in map {
        rev_map.entry(v).or_default().push(k);
    }

    let target = 0x125F04080;
    let range = (0, 16);
    let max_lv = 5;
    let max_size = max_lv * 2 + 9;

    let mut out = vec![];

    path_find_helpers(rev_map, target, &mut out, range, max_lv, &startpoints).unwrap();

    let mut aout = vec![];
    for v in out.chunks(max_size) {
        let (start, path) = v.split_at(8);
        let start = usize::from_le_bytes(start.try_into().unwrap()) - 0x104B18000;
        aout.extend(start.to_le_bytes());
        aout.extend(path)
    }

    assert_eq!(
        aout,
        [
            40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 16, 0, 0, 0, 0, 0, 101, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
            0, 0, 101, 0, 0, 40, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 0, 0, 0, 0
        ]
    );
}
