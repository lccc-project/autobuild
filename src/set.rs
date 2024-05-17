use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    iter::FusedIterator,
};

use serde::de::DeserializeSeed;

#[derive(Clone)]
pub struct OrderedSet<K, S = crate::map::RandomState<2, 4>>(crate::map::OrderedMap<K, (), S>);

impl<K: core::fmt::Debug, S> core::fmt::Debug for OrderedSet<K, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<K> OrderedSet<K> {
    pub fn new() -> Self {
        Self(crate::map::OrderedMap::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(crate::map::OrderedMap::with_capacity(cap))
    }
}

impl<K, S> OrderedSet<K, S> {
    pub const fn with_hasher(hasher: S) -> Self {
        Self(crate::map::OrderedMap::with_hasher(hasher))
    }

    pub fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self(crate::map::OrderedMap::with_capacity_and_hasher(
            cap, hasher,
        ))
    }

    pub fn iter(&self) -> Iter<K> {
        Iter(self.0.iter())
    }
}

impl<K, S: Default> Default for OrderedSet<K, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, S> IntoIterator for OrderedSet<K, S> {
    type Item = K;
    type IntoIter = IntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<K: Eq + Hash, S: BuildHasher> OrderedSet<K, S> {
    pub fn insert(&mut self, val: K) -> bool {
        self.0.insert(val, ()).is_none()
    }

    pub fn contains<Q: Eq + Hash + ?Sized>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.0.contains_key(key)
    }

    pub fn remove<Q: Eq + Hash + ?Sized>(&mut self, key: &Q) -> Option<K>
    where
        K: Borrow<Q>,
    {
        self.0.remove_entry(key).map(|(i, _)| i)
    }
}

pub struct IntoIter<K>(crate::map::IntoIter<K, ()>);

impl<K> Iterator for IntoIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, _)| i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K> ExactSizeIterator for IntoIter<K> {}

impl<K> FusedIterator for IntoIter<K> {}

impl<K> DoubleEndedIterator for IntoIter<K> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(i, _)| i)
    }
}

impl<'a, K, S> IntoIterator for &'a OrderedSet<K, S> {
    type IntoIter = Iter<'a, K>;
    type Item = &'a K;

    fn into_iter(self) -> Iter<'a, K> {
        self.iter()
    }
}

pub struct Iter<'a, K>(crate::map::Iter<'a, K, ()>);

impl<'a, K> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, _)| i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K> ExactSizeIterator for Iter<'a, K> {}

impl<'a, K> FusedIterator for Iter<'a, K> {}

impl<'a, K> DoubleEndedIterator for Iter<'a, K> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(i, _)| i)
    }
}

impl<K: Eq + Hash, S: BuildHasher> Extend<K> for OrderedSet<K, S> {
    fn extend<T: IntoIterator<Item = K>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(|i| (i, ())))
    }
}

impl<'a, K: Eq + Hash + Clone, S: BuildHasher> Extend<&'a K> for OrderedSet<K, S> {
    fn extend<T: IntoIterator<Item = &'a K>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(|i| (i, &())))
    }
}

impl<K: Eq + Hash, S: BuildHasher + Default> FromIterator<K> for OrderedSet<K, S> {
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (len, _) = iter.size_hint();

        let mut set = Self::with_capacity_and_hasher(len, S::default());

        set.extend(iter);

        set
    }
}

impl<'a, K: Eq + Hash + Clone, S: BuildHasher + Default> FromIterator<&'a K> for OrderedSet<K, S> {
    fn from_iter<T: IntoIterator<Item = &'a K>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (len, _) = iter.size_hint();

        let mut set = Self::with_capacity_and_hasher(len, S::default());

        set.extend(iter);

        set
    }
}

impl<K: serde::Serialize, S> serde::Serialize for OrderedSet<K, S> {
    fn serialize<__S: serde::ser::Serializer>(&self, ser: __S) -> Result<__S::Ok, __S::Error> {
        ser.collect_seq(self)
    }
}

impl<'de, K: serde::Deserialize<'de> + Hash + Eq, S: BuildHasher + Default> serde::Deserialize<'de>
    for OrderedSet<K, S>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::WithHasher::new(S::default()).deserialize(deserializer)
    }
}

pub mod de;
