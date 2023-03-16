use core::{cmp::Ordering, ops::Bound::Included};

use std::{collections::BTreeMap, io, ptr};

use crate::consts::{Address, CHUNK_SIZE, LV1_OUT_SIZE, LV2_OUT_SIZE, MAX_DEPTH, POINTER_SIZE};

use super::{
    error::{Error, Result},
    vmmap::{VirtualMemoryRead, VirtualQuery},
};

#[derive(Default)]
pub struct Pointer {
    map: BTreeMap<Address, Address>,
    inverse_map: BTreeMap<Address, Vec<Address>>,
    pointers: Vec<Address>,
}

impl Pointer {
    pub fn create_map<P, I, V>(&mut self, proc: &P, region: I) -> Result<()>
    where
        P: VirtualMemoryRead + Clone + Sync,
        V: VirtualQuery,
        I: Iterator<Item = V>,
    {
        let mut buf = vec![0; CHUNK_SIZE];

        let region = region.map(|m| (m.start(), m.size())).collect::<Vec<_>>();

        for (start, size) in &region {
            for off in (0..*size).step_by(CHUNK_SIZE) {
                let size = proc
                    .read_at(start + off, buf.as_mut_slice())
                    .map_err(Error::Vmmap)?;

                let buf = &buf[..size];

                for (o, buf) in buf.windows(POINTER_SIZE).enumerate() {
                    let addr = start + off + o;
                    let mut arr = [0; POINTER_SIZE];
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
                        self.map.insert(addr, out_addr);
                    }
                }
            }
        }

        for (&k, &v) in &self.map {
            self.inverse_map.entry(v).or_default().push(k);
        }

        self.pointers.extend(self.map.keys());

        Ok(())
    }

    fn walk_down_lv2<W, V>(
        &self,
        (addr, watchs): (Address, &[V]),
        (lrange, urange): (u16, u16),
        (max_levels, level): (u8, u8),
        startpoints: &[Address],
        writer: &mut W,
        (tmp_v, tmp_s): (&mut Vec<i16>, &mut [u8; LV2_OUT_SIZE]),
    ) -> Result<()>
    where
        W: io::Write,
        V: VirtualQuery,
    {
        let min = addr.saturating_sub(urange as _);
        let max = addr.saturating_add(lrange as _);

        let idx = startpoints.binary_search(&min).unwrap_or_else(|x| x);

        let mut iter = startpoints.iter().skip(idx).copied().take_while(|&v| v <= max);

        let mut m = iter.next();

        for e in iter {
            let off = signed_diff(addr, e).abs();
            if off < signed_diff(addr, m.unwrap()).abs() {
                m = Some(e);
            }
        }

        // TODO 三级包括三级以下长度的路径应该直接忽略？
        // TODO 允许只保留指定offset结束的路径？
        // TODO 允许保留除了基址以外的地址？
        // TODO 如果基址模块只有一个可以去掉name的16个字节以及避免这个循环
        if let Some(e) = m {
            let off = signed_diff(addr, e);
            tmp_v.push(off);
            for watch in watchs {
                let (start, end, name) = (watch.start(), watch.end(), watch.name());
                if (start..end).contains(&e) {
                    // 0..16 是名字， 16..20 是offset， 20..last b'e' 每两字节转换为i16然后反转是路径
                    let mut tmp = [0; 16];
                    unsafe {
                        let (l, _) = tmp.split_at_mut_unchecked(name.len());
                        ptr::copy_nonoverlapping(name.as_bytes().as_ptr(), l.as_mut_ptr(), l.len());
                        let (l, r) = tmp_s.split_at_mut_unchecked(tmp.len());
                        ptr::copy_nonoverlapping(tmp.as_ptr(), l.as_mut_ptr(), l.len());
                        let off = ((e - start) as u32).to_le_bytes();
                        let (l, r) = r.split_at_mut_unchecked(off.len());
                        ptr::copy_nonoverlapping(off.as_ptr(), l.as_mut_ptr(), l.len());
                        let path = core::slice::from_raw_parts(tmp_v.as_ptr() as *const u8, tmp_v.len() * 2);
                        let (l, r) = r.split_at_mut_unchecked(path.len());
                        ptr::copy_nonoverlapping(path.as_ptr(), l.as_mut_ptr(), l.len());
                        let (l, _) = r.split_at_mut_unchecked(1);
                        ptr::copy_nonoverlapping([b'e'].as_ptr(), l.as_mut_ptr(), l.len());
                    }
                    writer.write_all(tmp_s).map_err(Error::Io)?;
                    tmp_s.fill(0);
                }
            }
            tmp_v.pop();
        }

        if level < max_levels {
            for (&k, vec) in self.inverse_map.range((Included(&min), Included(&max))) {
                let off = signed_diff(addr, k);
                tmp_v.push(off);
                for &v in vec {
                    self.walk_down_lv2(
                        (v, watchs),
                        (lrange, urange),
                        (max_levels, level + 1),
                        startpoints,
                        writer,
                        (tmp_v, tmp_s),
                    )?;
                }
                tmp_v.pop();
            }
        }

        Ok(())
    }

    fn walk_down_lv1<W, V>(
        &self,
        (addr, watch): (Address, &V),
        (lrange, urange): (u16, u16),
        (max_levels, level): (u8, u8),
        startpoints: &[Address],
        writer: &mut W,
        (tmp_v, tmp_s): (&mut Vec<i16>, &mut [u8; LV1_OUT_SIZE]),
    ) -> Result<()>
    where
        W: io::Write,
        V: VirtualQuery,
    {
        let min = addr.saturating_sub(urange as _);
        let max = addr.saturating_add(lrange as _);

        let idx = startpoints.binary_search(&min).unwrap_or_else(|x| x);

        let mut iter = startpoints.iter().skip(idx).copied().take_while(|&v| v <= max);

        let mut m = iter.next();

        for e in iter {
            let off = signed_diff(addr, e).abs();
            if off < signed_diff(addr, m.unwrap()).abs() {
                m = Some(e);
            }
        }

        // TODO 三级包括三级以下长度的路径应该直接忽略？
        // TODO 允许只保留指定offset结束的路径？
        // TODO 允许保留除了基址以外的地址？
        // TODO 如果基址模块只有一个可以去掉name的16个字节以及避免这个循环
        if let Some(e) = m {
            let off = signed_diff(addr, e);
            tmp_v.push(off);

            let (start, end) = (watch.start(), watch.end());
            if (start..end).contains(&e) {
                // 16..20 是offset， 20..last b'e' 每两字节转换为i16然后反转是路径
                unsafe {
                    let off = ((e - start) as u32).to_le_bytes();
                    let (l, r) = tmp_s.split_at_mut_unchecked(off.len());
                    ptr::copy_nonoverlapping(off.as_ptr(), l.as_mut_ptr(), l.len());
                    let path = core::slice::from_raw_parts(tmp_v.as_ptr() as *const u8, tmp_v.len() * 2);
                    let (l, r) = r.split_at_mut_unchecked(path.len());
                    ptr::copy_nonoverlapping(path.as_ptr(), l.as_mut_ptr(), l.len());
                    let (l, _) = r.split_at_mut_unchecked(1);
                    ptr::copy_nonoverlapping([b'e'].as_ptr(), l.as_mut_ptr(), l.len());
                }
                writer.write_all(tmp_s).map_err(Error::Io)?;
                tmp_s.fill(0);
            }
            tmp_v.pop();
        }

        if level < max_levels {
            for (&k, vec) in self.inverse_map.range((Included(&min), Included(&max))) {
                let off = signed_diff(addr, k);
                tmp_v.push(off);
                for &v in vec {
                    self.walk_down_lv1(
                        (v, watch),
                        (lrange, urange),
                        (max_levels, level + 1),
                        startpoints,
                        writer,
                        (tmp_v, tmp_s),
                    )?;
                }
                tmp_v.pop();
            }
        }

        Ok(())
    }

    pub fn find_path<W, I, V>(
        &self,
        bases: &[V],
        range: (u16, u16),
        max_depth: u8,
        search_for: I,
        entry_points: Option<&Vec<Address>>,
    ) -> Result<()>
    where
        W: io::Write,
        I: Iterator<Item = (W, Address)>,
        V: VirtualQuery,
    {
        // 记录解析模式 1 为 lv1 单个模块使用的扫描，记录扫描的名字。 2为lv2，使用lv2什么也没记录
        let mut magic = [0_u8; 20];

        if bases.is_empty() {
            return Err("bases is empty".into());
        } else if bases.len() == 1 {
            let base = bases.get(0).ok_or("get base error")?;
            let (l, r) = magic.split_at_mut(1);
            l.copy_from_slice(&[1]);
            let (l, r) = r.split_at_mut(1);
            l.copy_from_slice(&max_depth.to_le_bytes());
            let name = base.name().as_bytes();
            let (l, _) = r.split_at_mut(name.len());
            l.copy_from_slice(name);

            match entry_points {
                Some(entry) => {
                    for (mut writer, target) in search_for {
                        writer.write_all(&magic)?;
                        self.walk_down_lv1(
                            (target, base),
                            range,
                            (max_depth, 1),
                            entry,
                            &mut writer,
                            (&mut Vec::with_capacity(MAX_DEPTH as _), &mut [0; LV1_OUT_SIZE]),
                        )?;
                    }
                }
                None => {
                    for (mut writer, target) in search_for {
                        writer.write_all(&magic)?;
                        self.walk_down_lv1(
                            (target, base),
                            range,
                            (max_depth, 1),
                            &self.pointers,
                            &mut writer,
                            (&mut Vec::with_capacity(MAX_DEPTH as _), &mut [0; LV1_OUT_SIZE]),
                        )?;
                    }
                }
            }
        } else {
            let (l, r) = magic.split_at_mut(1);
            l.copy_from_slice(&[2]);
            let (l, _) = r.split_at_mut(1);
            l.copy_from_slice(&max_depth.to_le_bytes());

            match entry_points {
                Some(entry) => {
                    for (mut writer, target) in search_for {
                        writer.write_all(&magic)?;
                        self.walk_down_lv2(
                            (target, bases),
                            range,
                            (max_depth, 1),
                            entry,
                            &mut writer,
                            (&mut Vec::with_capacity(MAX_DEPTH as _), &mut [0; LV2_OUT_SIZE]),
                        )?;
                    }
                }
                None => {
                    for (mut writer, target) in search_for {
                        writer.write_all(&magic)?;
                        self.walk_down_lv2(
                            (target, bases),
                            range,
                            (max_depth, 1),
                            &self.pointers,
                            &mut writer,
                            (&mut Vec::with_capacity(MAX_DEPTH as _), &mut [0; LV2_OUT_SIZE]),
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[inline(always)]
pub fn signed_diff(a: Address, b: Address) -> i16 {
    a.checked_sub(b)
        .map(|a| a as i16)
        .unwrap_or_else(|| -((b - a) as i16))
}
