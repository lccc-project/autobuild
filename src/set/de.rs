use serde::de::{DeserializeSeed, SeqAccess, Visitor};

use core::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct WithHasher<E, S>(E, S);

impl<E, S> WithHasher<PhantomData<E>, S> {
    pub const fn new(hasher: S) -> Self {
        Self(PhantomData, hasher)
    }
}

impl<E, S> WithHasher<E, S> {
    #[allow(dead_code)]
    pub const fn new_seeded(elem_seed: E, hasher: S) -> Self {
        Self(elem_seed, hasher)
    }
}

impl<'de, E: DeserializeSeed<'de> + Clone, S: BuildHasher> DeserializeSeed<'de> for WithHasher<E, S>
where
    E::Value: Hash + Eq,
{
    type Value = super::OrderedSet<E::Value, S>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(WithHasherVisitor(self.0, self.1))
    }
}

struct WithHasherVisitor<E, S>(E, S);

impl<'de, E: DeserializeSeed<'de> + Clone, S: BuildHasher> Visitor<'de> for WithHasherVisitor<E, S>
where
    E::Value: Hash + Eq,
{
    type Value = super::OrderedSet<E::Value, S>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a set")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let cap = seq.size_hint().unwrap_or(0);

        let mut ret = super::OrderedSet::with_capacity_and_hasher(cap, self.1);

        while let Some(val) = seq.next_element_seed(self.0.clone())? {
            ret.insert(val);
        }
        Ok(ret)
    }
}
