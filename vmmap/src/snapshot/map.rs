use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Bound, Range},
};

struct RangeWrapper<T>(Range<T>);

impl<T: PartialEq> PartialEq for RangeWrapper<T> {
    fn eq(&self, other: &RangeWrapper<T>) -> bool {
        self.0.start == other.0.start
    }
}

impl<T: Eq> Eq for RangeWrapper<T> {}

impl<T: Ord> Ord for RangeWrapper<T> {
    fn cmp(&self, other: &RangeWrapper<T>) -> Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl<T: PartialOrd> PartialOrd for RangeWrapper<T> {
    fn partial_cmp(&self, other: &RangeWrapper<T>) -> Option<Ordering> {
        self.0.start.partial_cmp(&other.0.start)
    }
}

#[derive(Default)]
pub struct RangeMap<K, V>(BTreeMap<RangeWrapper<K>, V>);

impl<K, V> RangeMap<K, V>
where
    K: Ord + Copy,
{
    #[inline]
    pub fn get_key_value(&self, point: K) -> Option<(&Range<K>, &V)> {
        let start = RangeWrapper(point..point);
        self.0
            .range((Bound::Unbounded, Bound::Included(start)))
            .next_back()
            .filter(|(range, _)| range.0.contains(&point))
            .map(|(range, value)| (&range.0, value))
    }
}

impl<K, V> RangeMap<K, V>
where
    K: Ord + Copy,
    V: Eq + Clone,
{
    #[inline]
    pub fn insert(&mut self, key: Range<K>, value: V) {
        assert!(key.start <= key.end);
        self.0.insert(RangeWrapper(key), value);
    }
}

impl<K, V> Extend<(Range<K>, V)> for RangeMap<K, V>
where
    K: Ord + Copy,
    V: Eq + Clone,
{
    #[inline]
    fn extend<T: IntoIterator<Item = (Range<K>, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.insert(k, v);
        })
    }
}
