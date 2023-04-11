use std::{collections::BTreeMap, io, path::PathBuf};

use crate::{
    consts::{Address, BIN_CONFIG},
    engine::PointerSeacher,
    Result,
};

pub struct PathFindEngine<'a, W> {
    target: Address,
    depth: usize,
    offset: (usize, usize),
    pout: &'a mut W,
    startpoints: Vec<Address>,
    engine: PointerSeacher,
}

impl<W> PathFindEngine<'_, W>
where
    W: io::Write,
{
    pub fn find_pointer_path(self) -> Result<()> {
        let PathFindEngine { target, depth, offset, pout: out, engine, startpoints } = self;
        let size = depth * 2 + 9;
        out.write_all(&size.to_le_bytes())?;
        engine.path_find_helpers(target, out, offset, depth, size, &startpoints)?;
        Ok(())
    }
}

pub enum Filter {
    Start(Vec<Address>),
    Range(Vec<(usize, usize, PathBuf)>),
}

pub struct PathFindParams<'a, W> {
    pub target: Address,
    pub depth: usize,
    pub offset: (usize, usize),
    pub pout: &'a mut W,
    pub mout: &'a mut W,
    pub maps: Vec<(usize, usize, PathBuf)>,
    pub filter: Option<Filter>,
}

pub fn ptrsx_init_engine<R: io::Read, W: io::Write>(
    mut p_read: R,
    params: PathFindParams<'_, W>,
) -> Result<PathFindEngine<W>> {
    let ptrs: BTreeMap<Address, Address> = bincode::decode_from_std_read(&mut p_read, BIN_CONFIG)?;

    let PathFindParams { target, depth, offset, pout, mout, maps, filter } = params;

    let startpoints = match filter {
        Some(fi) => match fi {
            Filter::Start(points) => {
                let range = maps
                    .into_iter()
                    .filter(|&(start, end, _)| points.iter().any(|a| (start..end).contains(a)))
                    .collect::<Vec<_>>();
                bincode::encode_into_std_write(range, mout, BIN_CONFIG)?;
                points
            }
            Filter::Range(range) => {
                bincode::encode_into_std_write(&range, mout, BIN_CONFIG)?;
                ptrs.keys()
                    .copied()
                    .filter(|a| range.iter().any(|&(start, end, _)| (start..end).contains(a)))
                    .collect()
            }
        },
        None => {
            bincode::encode_into_std_write(maps, mout, BIN_CONFIG)?;
            ptrs.keys().copied().collect()
        }
    };

    let mut map: BTreeMap<Address, Vec<Address>> = BTreeMap::new();
    for (k, v) in ptrs {
        map.entry(v).or_default().push(k);
    }

    Ok(PathFindEngine {
        target,
        depth,
        offset,
        pout,
        startpoints,
        engine: PointerSeacher(map),
    })
}
