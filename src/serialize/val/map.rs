use hashbrown::raw::RawTable;

struct OrderedMapImpl<K, V> {
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
}
