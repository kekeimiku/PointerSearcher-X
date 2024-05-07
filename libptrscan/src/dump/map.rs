use core::{
    cmp::Ordering,
    ops::{Bound, Range},
};
use std::collections::{btree_map, BTreeMap};

struct RangeWrapper<T>(Range<T>);

impl<T: PartialEq> PartialEq for RangeWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.start == other.0.start
    }
}

impl<T: Eq> Eq for RangeWrapper<T> {}

impl<T: Ord> Ord for RangeWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl<T: PartialOrd> PartialOrd for RangeWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.start.partial_cmp(&other.0.start)
    }
}

pub struct ModuleMap<K, V>(BTreeMap<RangeWrapper<K>, V>);

impl<K, V> Default for ModuleMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> ModuleMap<K, V> {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<K, V> ModuleMap<K, V>
where
    K: Ord + Copy,
{
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter(self.0.iter())
    }

    pub fn get_key_value_by_point(&self, point: K) -> Option<(&Range<K>, &V)> {
        let start = RangeWrapper(point..point);
        self.0
            .range((Bound::Unbounded, Bound::Included(start)))
            .next_back()
            .filter(|(k, _)| k.0.contains(&point))
            .map(|(k, v)| (&k.0, v))
    }

    pub fn insert(&mut self, key: Range<K>, value: V) -> Option<V> {
        assert!(key.start <= key.end);
        self.0.insert(RangeWrapper(key), value)
    }
}

impl<K, V> Extend<(Range<K>, V)> for ModuleMap<K, V>
where
    K: Ord + Copy,
{
    fn extend<T: IntoIterator<Item = (Range<K>, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.insert(k, v);
        })
    }
}

impl<K, V> FromIterator<(Range<K>, V)> for ModuleMap<K, V>
where
    K: Ord + Copy,
{
    fn from_iter<T: IntoIterator<Item = (Range<K>, V)>>(iter: T) -> Self {
        let mut map = ModuleMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

pub struct IntoIter<K, V>(btree_map::IntoIter<RangeWrapper<K>, V>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (Range<K>, V);

    fn next(&mut self) -> Option<(Range<K>, V)> {
        self.0.next().map(|(k, v)| (k.0, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> IntoIterator for ModuleMap<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (Range<K>, V);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

pub struct Iter<'a, K, V>(btree_map::Iter<'a, RangeWrapper<K>, V>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a Range<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (&k.0, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
