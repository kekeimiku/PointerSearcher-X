use std::path::PathBuf;

use consts::Address;

#[derive(Default, Debug, Clone)]
pub struct Map {
    pub start: Address,
    pub end: Address,
    pub path: PathBuf,
}

pub struct MapIter<'a>(pub core::str::Lines<'a>);

impl Iterator for MapIter<'_> {
    type Item = Map;

    fn next(&mut self) -> Option<Self::Item> {
        let mut split = self.0.next()?.split('-');
        let start = split.next()?.parse().ok()?;
        let end = split.next()?.parse().ok()?;
        let path = PathBuf::from(split.next()?);
        Some(Map { start, end, path })
    }
}
