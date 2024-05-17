use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use core::iter::{Extend, FromIterator, FusedIterator};
use std::marker::PhantomData;
use std::ops::Index;

mod raw;

use raw::OrderedMapImpl;

pub use lccc_siphash::SipHasher;
use serde::de::{DeserializeSeed, Visitor};

use crate::rand::Rand;

#[derive(Clone, Debug)]
pub struct RandomState<const C: usize, const D: usize>(u64, u64);

impl<const C: usize, const D: usize> Default for RandomState<C, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const C: usize, const D: usize> RandomState<C, D> {
    pub fn new() -> RandomState<C, D> {
        let mut rand = Rand::init();

        Self::from_generator(&mut rand)
    }

    pub fn from_generator(gen: &mut Rand) -> RandomState<C, D> {
        Self(gen.gen(), gen.gen())
    }
}

impl<const C: usize, const D: usize> BuildHasher for RandomState<C, D> {
    type Hasher = SipHasher<C, D>;

    fn build_hasher(&self) -> Self::Hasher {
        SipHasher::new_with_keys(self.0, self.1)
    }
}

/// A Hash table based collection that iterates in insertion order
#[derive(Clone)]
pub struct OrderedMap<K, V, S = RandomState<2, 4>> {
    inner: OrderedMapImpl<K, V>,
    hasher: S,
}

impl<K: core::fmt::Debug, V: core::fmt::Debug, S> core::fmt::Debug for OrderedMap<K, V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<K, V, S: Default> Default for OrderedMap<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: OrderedMapImpl::new(),
            hasher: RandomState::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: OrderedMapImpl::with_capacity(cap),
            hasher: RandomState::new(),
        }
    }
}

impl<K, V, S> OrderedMap<K, V, S> {
    pub const fn with_hasher(hasher: S) -> Self {
        Self {
            inner: OrderedMapImpl::new(),
            hasher,
        }
    }

    pub fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            inner: OrderedMapImpl::with_capacity(cap),
            hasher,
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> OrderedMap<K, V, S> {
    fn hash_key<Q: Hash + ?Sized>(hasher: &S, key: &Q) -> u64 {
        let mut hasher = hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn key_eq<Q: Hash + Eq + ?Sized>(key: &Q) -> impl FnMut(&K) -> bool + '_
    where
        K: Borrow<Q>,
    {
        move |k| <K as Borrow<Q>>::borrow(k) == key
    }

    pub fn reserve(&mut self, additional: usize) {
        let Self { inner, hasher } = self;

        inner.reserve(additional, |key| Self::hash_key(hasher, key));
    }

    pub fn contains_key<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn get<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let hash = Self::hash_key(&self.hasher, key);
        self.inner.get(hash, Self::key_eq(key)).map(|(_, v)| v)
    }

    pub fn get_key_value<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
    {
        let hash = Self::hash_key(&self.hasher, key);

        self.inner.get(hash, Self::key_eq(key)).map(|(k, v)| (k, v))
    }

    pub fn get_mut<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let hash = Self::hash_key(&self.hasher, key);
        self.inner.get_mut(hash, Self::key_eq(key)).map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hasher = &self.hasher;
        let hash = Self::hash_key(hasher, &key);
        let (entry, val) =
            self.inner
                .insert_entry(hash, key, value, |key| Self::hash_key(hasher, key), K::eq);

        if let Some((_, val)) = val {
            Some(core::mem::replace(&mut entry.1, val))
        } else {
            None
        }
    }

    pub fn get_or_insert_mut(&mut self, key: K, value: V) -> &mut V {
        let hasher = &self.hasher;
        let hash = Self::hash_key(hasher, &key);
        let (entry, _) =
            self.inner
                .insert_entry(hash, key, value, |key| Self::hash_key(hasher, key), K::eq);

        &mut entry.1
    }

    pub fn get_or_insert_with_mut<D: FnOnce(&K) -> V>(&mut self, key: K, val: D) -> &mut V {
        let hasher = &self.hasher;
        let hash = Self::hash_key(hasher, &key);
        let (entry, _) =
            self.inner
                .insert_entry_with(hash, key, val, |key| Self::hash_key(hasher, key), K::eq);

        &mut entry.1
    }

    pub fn remove<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let hash = Self::hash_key(&self.hasher, key);
        self.inner
            .remove(hash, |k| k.borrow() == key)
            .map(|(_, val)| val)
    }

    pub fn remove_entry<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
    {
        let hash = Self::hash_key(&self.hasher, key);
        self.inner.remove(hash, |k| k.borrow() == key)
    }
}

impl<K: Hash + Eq, Q: Hash + Eq + ?Sized, V, S: BuildHasher> Index<&Q> for OrderedMap<K, V, S>
where
    K: Borrow<Q>,
{
    type Output = V;

    #[track_caller]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no such key in table")
    }
}

impl<K, V, S> OrderedMap<K, V, S> {
    pub fn iter(&self) -> Iter<K, V> {
        Iter(self.inner.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut(self.inner.iter_mut())
    }
}

impl<K, V, S> IntoIterator for OrderedMap<K, V, S> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.inner.into_iter())
    }
}

impl<'a, K, V, S> IntoIterator for &'a OrderedMap<K, V, S> {
    type IntoIter = Iter<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OrderedMap<K, V, S> {
    type IntoIter = IterMut<'a, K, V>;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Hash + Eq, V, S: Default + BuildHasher> FromIterator<(K, V)> for OrderedMap<K, V, S> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::default();

        map.extend(iter);

        map
    }
}

impl<'a, 'b, K: Hash + Eq + Clone, V: Clone, S: Default + BuildHasher> FromIterator<(&'a K, &'b V)>
    for OrderedMap<K, V, S>
{
    fn from_iter<T: IntoIterator<Item = (&'a K, &'b V)>>(iter: T) -> Self {
        let mut map = Self::default();

        map.extend(iter);

        map
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Extend<(K, V)> for OrderedMap<K, V, S> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let mut iter = iter.into_iter();
        self.reserve(iter.size_hint().0);

        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, 'b, K: Hash + Eq + Clone, V: Clone, S: BuildHasher> Extend<(&'a K, &'b V)>
    for OrderedMap<K, V, S>
{
    fn extend<T: IntoIterator<Item = (&'a K, &'b V)>>(&mut self, iter: T) {
        let mut iter = iter.into_iter();
        self.reserve(iter.size_hint().0);

        for (k, v) in iter {
            self.insert(k.clone(), v.clone());
        }
    }
}

pub struct IntoIter<K, V>(raw::IntoIter<K, V>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}
impl<K, V> FusedIterator for IntoIter<K, V> {}

pub struct Iter<'a, K, V>(raw::Iter<'a, K, V>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, v)| (k, v))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

pub struct IterMut<'a, K, V>(raw::IterMut<'a, K, V>);

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (&*k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, v)| (&*k, v))
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}

impl<K: serde::ser::Serialize + Hash + Eq, V: serde::ser::Serialize, S> serde::ser::Serialize
    for OrderedMap<K, V, S>
{
    fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut ser = serializer.serialize_map(Some(self.len()))?;

        for (key, value) in self {
            ser.serialize_entry(key, value)?;
        }

        ser.end()
    }
}

impl<
        'de,
        K: serde::de::Deserialize<'de> + Hash + Eq,
        V: serde::de::Deserialize<'de>,
        S: BuildHasher + Default,
    > serde::de::Deserialize<'de> for OrderedMap<K, V, S>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::WithHasher::new(S::default()).deserialize(deserializer)
    }
}

pub mod de;
