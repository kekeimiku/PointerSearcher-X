// RangeMap 储存模块信息（不合并区间），可以通过一个点来查询模块名以及范围
// RangeSet 储存内存信息（合并区间），可以通过一个点来查询范围

use core::{
    cmp::{max, min, Ordering},
    ops::{Bound, Range},
};

extern crate alloc;
use alloc::collections::{btree_map, btree_set, BTreeMap, BTreeSet};

#[derive(Clone)]
struct RangeWrapper<T>(Range<T>);

impl<T: PartialEq> PartialEq for RangeWrapper<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.start == other.0.start
    }
}

impl<T: Eq> Eq for RangeWrapper<T> {}

impl<T: Ord> Ord for RangeWrapper<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl<T: PartialOrd> PartialOrd for RangeWrapper<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.start.partial_cmp(&other.0.start)
    }
}

trait RangeExt<T> {
    fn overlaps(&self, other: &Self) -> bool;
    fn touches(&self, other: &Self) -> bool;
}

impl<T> RangeExt<T> for Range<T>
where
    T: Ord,
{
    #[inline]
    fn overlaps(&self, other: &Self) -> bool {
        max(&self.start, &other.start) < min(&self.end, &other.end)
    }

    #[inline]
    fn touches(&self, other: &Self) -> bool {
        max(&self.start, &other.start) <= min(&self.end, &other.end)
    }
}

#[derive(Clone)]
pub struct RangeMap<K, V>(BTreeMap<RangeWrapper<K>, V>);

impl<K, V> RangeMap<K, V>
where
    K: Ord + Clone,
{
    pub fn get_key_value_by_point(&self, point: &K) -> Option<(&Range<K>, &V)> {
        let start = RangeWrapper(point.clone()..point.clone());
        self.0
            .range((Bound::Unbounded, Bound::Included(start)))
            .next_back()
            .filter(|(k, _)| k.0.contains(point))
            .map(|(k, v)| (&k.0, v))
    }

    pub fn insert(&mut self, key: Range<K>, value: V) -> Option<V> {
        assert!(key.start <= key.end);
        self.0.insert(RangeWrapper(key), value)
    }
}

impl<K, V> Default for RangeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> RangeMap<K, V> {
    #[inline]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn iter(&self) -> RangeMapIter<'_, K, V> {
        RangeMapIter(self.0.iter())
    }
}

impl<K, V> Extend<(Range<K>, V)> for RangeMap<K, V>
where
    K: Ord + Clone,
{
    #[inline]
    fn extend<T: IntoIterator<Item = (Range<K>, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.insert(k, v);
        })
    }
}

impl<K, V> FromIterator<(Range<K>, V)> for RangeMap<K, V>
where
    K: Ord + Clone,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (Range<K>, V)>>(iter: T) -> Self {
        let mut map = RangeMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

pub struct RangeMapIntoIter<K, V>(btree_map::IntoIter<RangeWrapper<K>, V>);

impl<K, V> Iterator for RangeMapIntoIter<K, V> {
    type Item = (Range<K>, V);

    #[inline]
    fn next(&mut self) -> Option<(Range<K>, V)> {
        self.0.next().map(|(k, v)| (k.0, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> IntoIterator for RangeMap<K, V> {
    type IntoIter = RangeMapIntoIter<K, V>;
    type Item = (Range<K>, V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RangeMapIntoIter(self.0.into_iter())
    }
}

pub struct RangeMapIter<'a, K, V>(btree_map::Iter<'a, RangeWrapper<K>, V>);

impl<'a, K, V> Iterator for RangeMapIter<'a, K, V> {
    type Item = (&'a Range<K>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (&k.0, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct RangeSet<K>(BTreeSet<RangeWrapper<K>>);

impl<K> RangeSet<K>
where
    K: Ord + Clone,
{
    fn adjust_touching_ranges_for_insert(
        &mut self,
        stored_range: RangeWrapper<K>,
        new_range: &mut Range<K>,
    ) {
        new_range.start = min(&new_range.start, &stored_range.0.start).clone();
        new_range.end = max(&new_range.end, &stored_range.0.end).clone();
        self.0.remove(&stored_range);
        if new_range.overlaps(&stored_range.0) {
            self.0.remove(&stored_range);
            if stored_range.0.start < new_range.start {
                self.0
                    .insert(RangeWrapper(stored_range.0.start..new_range.start.clone()));
            }
            if stored_range.0.end > new_range.end {
                self.0
                    .insert(RangeWrapper(new_range.end.clone()..stored_range.0.end));
            }
        }
    }

    pub fn insert(&mut self, range: Range<K>) {
        assert!(range.start < range.end);

        let mut new_range = RangeWrapper(range);

        let mut candidates = self
            .0
            .range((Bound::Unbounded, Bound::Included(&new_range)))
            .rev()
            .take(2)
            .filter(|stored_range| stored_range.0.touches(&new_range.0));
        if let Some(mut candidate) = candidates.next() {
            if let Some(another_candidate) = candidates.next() {
                candidate = another_candidate;
            }
            self.adjust_touching_ranges_for_insert(candidate.clone(), &mut new_range.0);
        }

        let new_range_end_as_start = RangeWrapper(new_range.0.end.clone()..new_range.0.end.clone());
        while let Some(stored_range) = self
            .0
            .range((
                Bound::Included(&new_range),
                Bound::Included(&new_range_end_as_start),
            ))
            .next()
        {
            self.adjust_touching_ranges_for_insert(stored_range.clone(), &mut new_range.0);
        }

        self.0.insert(new_range);
    }

    pub fn get_range_by_point(&self, point: &K) -> Option<&Range<K>> {
        let start = RangeWrapper(point.clone()..point.clone());
        self.0
            .range((Bound::Unbounded, Bound::Included(start)))
            .next_back()
            .filter(|k| k.0.contains(point))
            .map(|k| &k.0)
    }
}

impl<K> Default for RangeSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> RangeSet<K> {
    #[inline]
    pub const fn new() -> Self {
        Self(BTreeSet::new())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn iter(&self) -> RangeSetIter<'_, K> {
        RangeSetIter(self.0.iter())
    }
}

pub struct RangeSetIntoIter<K>(btree_set::IntoIter<RangeWrapper<K>>);

impl<K> Iterator for RangeSetIntoIter<K> {
    type Item = Range<K>;

    #[inline]
    fn next(&mut self) -> Option<Range<K>> {
        self.0.next().map(|k| k.0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K> IntoIterator for RangeSet<K> {
    type IntoIter = RangeSetIntoIter<K>;
    type Item = Range<K>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RangeSetIntoIter(self.0.into_iter())
    }
}

impl<K> Extend<Range<K>> for RangeSet<K>
where
    K: Ord + Clone,
{
    #[inline]
    fn extend<T: IntoIterator<Item = Range<K>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |k| {
            self.insert(k);
        })
    }
}

impl<K> FromIterator<Range<K>> for RangeSet<K>
where
    K: Ord + Clone,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = Range<K>>>(iter: T) -> Self {
        let mut map = RangeSet::new();
        for k in iter {
            map.insert(k);
        }
        map
    }
}

pub struct RangeSetIter<'a, K>(btree_set::Iter<'a, RangeWrapper<K>>);

impl<'a, K> Iterator for RangeSetIter<'a, K> {
    type Item = &'a Range<K>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|k| &k.0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
