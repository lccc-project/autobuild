use serde::de::{Deserialize, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};

use core::marker::PhantomData;
use std::hash::{BuildHasher, Hash};

use crate::serialize::helpers::SeededPair;

#[derive(Clone)]
pub struct WithHasher<K, V, S>(K, V, S);

impl<'de, K: DeserializeSeed<'de> + Clone, V: DeserializeSeed<'de> + Clone, S: BuildHasher>
    DeserializeSeed<'de> for WithHasher<K, V, S>
where
    K::Value: Hash + Eq,
{
    type Value = super::OrderedMap<K::Value, V::Value, S>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(WithHasherVisitor(self.0, self.1, self.2))
    }
}

impl<K, V, S> WithHasher<PhantomData<K>, PhantomData<V>, S> {
    pub const fn new(hasher: S) -> Self {
        Self(PhantomData, PhantomData, hasher)
    }
}

impl<K, V, S> WithHasher<K, V, S> {
    pub const fn new_seeded(key_seed: K, val_seed: V, hasher: S) -> Self {
        Self(key_seed, val_seed, hasher)
    }
}

struct WithHasherVisitor<K, V, S>(K, V, S);

impl<'de, K: DeserializeSeed<'de> + Clone, V: DeserializeSeed<'de> + Clone, S: BuildHasher>
    Visitor<'de> for WithHasherVisitor<K, V, S>
where
    K::Value: Hash + Eq,
{
    type Value = super::OrderedMap<K::Value, V::Value, S>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let cap = map.size_hint().unwrap_or(0);

        let mut ret = super::OrderedMap::with_capacity_and_hasher(cap, self.2);

        while let Some((key, val)) = map.next_entry_seed(self.0.clone(), self.1.clone())? {
            ret.insert(key, val);
        }
        Ok(ret)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let cap = seq.size_hint().unwrap_or(0);

        let mut ret = super::OrderedMap::with_capacity_and_hasher(cap, self.2);

        while let Some((key, val)) =
            seq.next_element_seed(SeededPair(self.0.clone(), self.1.clone()))?
        {
            ret.insert(key, val);
        }
        Ok(ret)
    }
}
