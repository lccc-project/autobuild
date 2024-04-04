use hashbrown::raw::RawTable;

use std::iter::FusedIterator;

#[derive(Clone)]
pub struct OrderedMapImpl<K, V> {
    inner: RawTable<usize>,
    map: Vec<(K, V)>,
}

impl<K, V> OrderedMapImpl<K, V> {
    pub const fn new() -> Self {
        Self {
            inner: RawTable::new(),
            map: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RawTable::with_capacity(capacity),
            map: Vec::with_capacity(capacity),
        }
    }

    pub fn get<E: FnMut(&K) -> bool>(&self, hash: u64, mut eq: E) -> Option<&(K, V)> {
        let index = self.inner.get(hash, |val| eq(&self.map[*val].0)).copied()?;

        Some(unsafe { self.map.get_unchecked(index) })
    }

    pub fn get_mut<E: FnMut(&K) -> bool>(&mut self, hash: u64, mut eq: E) -> Option<&mut (K, V)> {
        let Self { inner, map } = self;
        let index = inner.get(hash, |val| eq(&map[*val].0)).copied()?;

        Some(unsafe { self.map.get_unchecked_mut(index) })
    }

    /// If `key` (according to `hasher` and `eq`) are not present in the table, inserts the result of `key_val_ctor` into the table and returns `true`
    /// Otherwise return `false`.
    /// In both cases, the returns a reference to the specified entry
    ///
    /// This does not update the existing entry if one is present.
    pub fn insert_entry_with<H: Fn(&K) -> u64, E: FnMut(&K, &K) -> bool, D: FnOnce(&K) -> V>(
        &mut self,
        hash: u64,
        key: K,
        val_ctor: D,
        hasher: H,
        mut eq: E,
    ) -> (&mut (K, V), bool) {
        let Self { inner, map } = self;
        match inner.find_or_find_insert_slot(
            hash,
            |elem| eq(&key, &map[*elem].0),
            |elem| hasher(&map[*elem].0),
        ) {
            Ok(bucket) => {
                // SAFETY: We only insert indecies in `map`, and `bucket` is still live for as long as `self` is
                let elem = unsafe { self.map.get_unchecked_mut(*bucket.as_ref()) };

                (elem, false)
            }
            Err(slot) => {
                let index = self.map.len();
                let val = val_ctor(&key);
                self.map.push((key, val));
                // SAFETY: We just got this `slot` and we haven't mutated `inner` since getting it
                unsafe {
                    self.inner.insert_in_slot(hash, slot, index);
                }

                (self.map.last_mut().unwrap(), true)
            }
        }
    }

    /// If `key` (according to `hasher` and `eq`) are not present in the table, inserts `key, val` into the table,
    /// Otherwise return `(key, value)`.
    /// In both cases, the function returns a reference to the specified entry
    ///
    /// This does not update the existing entry if one is present.
    pub fn insert_entry<H: Fn(&K) -> u64, E: FnMut(&K, &K) -> bool>(
        &mut self,
        hash: u64,
        key: K,
        val: V,
        hasher: H,
        mut eq: E,
    ) -> (&mut (K, V), Option<(K, V)>) {
        let Self { inner, map } = self;
        match inner.find_or_find_insert_slot(
            hash,
            |elem| eq(&key, &map[*elem].0),
            |elem| hasher(&map[*elem].0),
        ) {
            Ok(bucket) => {
                // SAFETY: We only insert indecies in `map`, and `bucket` is still live for as long as `self` is
                let elem = unsafe { self.map.get_unchecked_mut(*bucket.as_ref()) };

                (elem, Some((key, val)))
            }
            Err(slot) => {
                let index = self.map.len();
                self.map.push((key, val));
                // SAFETY: We just got this `slot` and we haven't mutated `inner` since getting it
                unsafe {
                    self.inner.insert_in_slot(hash, slot, index);
                }

                (self.map.last_mut().unwrap(), None)
            }
        }
    }

    pub fn remove<E: FnMut(&K) -> bool>(&mut self, hash: u64, mut eq: E) -> Option<(K, V)> {
        let Self { inner, map } = self;
        let index = inner.remove_entry(hash, |elem| eq(&map[*elem].0))?;

        for entry in unsafe { self.inner.iter() } {
            if unsafe { *entry.as_ref() } > index {
                *unsafe { entry.as_mut() } -= 1;
            }
        }

        Some(self.map.remove(index))
    }

    pub fn reserve<H: Fn(&K) -> u64>(&mut self, additional: usize, hasher: H) {
        let Self { inner, map } = self;
        inner.reserve(additional, |elem| hasher(&map[*elem].0));
        self.map.reserve(additional)
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity().min(self.map.capacity())
    }

    pub fn buckets(&self) -> usize {
        self.inner.buckets()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter(self.map.iter())
    }

    /// Note:
    /// Returns an iterator over `&mut (K,V)`.
    /// Care must be taken by consumers not to mutate `K` in any way that disturbs its hash. Otherwise, the table may produce incorrect results for lookup operations
    ///
    /// For this reason, a [`IntoIterator`] impl is not provided for `&mut Self`
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut(self.map.iter_mut())
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.map.clear();
    }
}

impl<K, V> IntoIterator for OrderedMapImpl<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.map.into_iter())
    }
}

pub struct IntoIter<K, V>(std::vec::IntoIter<(K, V)>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}
impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

pub struct Iter<'a, K, V>(core::slice::Iter<'a, (K, V)>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

pub struct IterMut<'a, K, V>(core::slice::IterMut<'a, (K, V)>);

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = &'a mut (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, K, V> FusedIterator for IterMut<'a, K, V> {}
