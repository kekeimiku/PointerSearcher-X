use std::path::PathBuf;

use consts::Address;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Map {
    pub start: Address,
    pub end: Address,
    pub path: PathBuf,
}

pub struct MapIter<'a>(pub core::str::Lines<'a>);

impl Iterator for MapIter<'_> {
    type Item = Map;

    fn next(&mut self) -> Option<Self::Item> {
        let mut split = self.0.next()?.splitn(3, ' ');
        let start = split.next()?.parse().ok()?;
        let end = split.next()?.parse().ok()?;
        let path = PathBuf::from(split.next()?);
        Some(Map { start, end, path })
    }
}

#[test]
fn test_parse_map() {
    let map = r#"94105692536832 94105694789632 /home/keke/.local/share/Steam/steamapps/common/Dead Cells/deadcells
139704257216512 139704259825664 /home/keke/.local/share/Steam/steamapps/common/Dead Cells/libhl.so"#;

    let map = MapIter(map.lines()).collect::<Vec<_>>();

    assert_eq!(
        map,
        vec![
            Map {
                start: 94105692536832,
                end: 94105694789632,
                path: PathBuf::from("/home/keke/.local/share/Steam/steamapps/common/Dead Cells/deadcells")
            },
            Map {
                start: 139704257216512,
                end: 139704259825664,
                path: PathBuf::from("/home/keke/.local/share/Steam/steamapps/common/Dead Cells/libhl.so")
            }
        ]
    );
}
