use std::{collections::BTreeMap, io, ops::Range};

use crate::{
    consts::{Address, BIN_CONFIG},
    engine::PointerSeacher,
    Result,
};

pub struct PathFindEngine<'a, W> {
    target: Address,
    depth: usize,
    offset: (usize, usize),
    out: &'a mut W,
    startpoints: Vec<Address>,
    engine: PointerSeacher,
}

impl<'a, W> PathFindEngine<'a, W>
where
    W: io::Write,
{
    pub fn find_pointer_path(self) -> Result<()> {
        let PathFindEngine { target, depth, offset, out, engine, startpoints } = self;
        let size = depth * 2 + 9;
        engine.path_find_helpers(target, out, offset, depth, size, &startpoints)?;
        Ok(())
    }
}

pub struct PathFindParams<'a, W> {
    pub target: Address,
    pub depth: usize,
    pub offset: (usize, usize),
    pub out: &'a mut W,
    pub filter: Option<Vec<Range<Address>>>,
    pub startpoints: Option<Vec<Address>>,
}

pub fn ptrsx_init_engine<R: io::Read, W: io::Write>(
    mut p_read: R,
    params: PathFindParams<'_, W>,
) -> Result<PathFindEngine<W>> {
    let ptrs: BTreeMap<Address, Address> = bincode::decode_from_std_read(&mut p_read, BIN_CONFIG)?;

    let PathFindParams { target, depth, offset, out, startpoints, filter } = params;

    let startpoints = match startpoints {
        Some(points) => points,
        None => match filter {
            Some(range) => ptrs
                .keys()
                .copied()
                .filter(|a| range.iter().any(|m| (m.start..m.end).contains(a)))
                .collect(),
            None => ptrs.keys().copied().collect(),
        },
    };

    let mut map: BTreeMap<Address, Vec<Address>> = BTreeMap::new();
    for (k, v) in ptrs {
        map.entry(v).or_default().push(k);
    }

    Ok(PathFindEngine {
        target,
        depth,
        offset,
        out,
        startpoints,
        engine: PointerSeacher(map),
    })
}
