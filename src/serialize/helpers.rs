use target_tuples::Target;

use serde::de::{Error, IgnoredAny, Unexpected};

struct TargetVisitor;

impl<'de> serde::de::Visitor<'de> for TargetVisitor {
    type Value = Target;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a (potentially non-canonical) target, in the form <arch>-<vendor>-<sys> (with vendor potentially omitted)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse()
            .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
    }
}

pub struct DeserializeTarget;

impl<'de> serde::de::DeserializeSeed<'de> for DeserializeTarget {
    type Value = Target;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TargetVisitor)
    }
}

#[derive(Copy, Clone)]
pub struct SeededPair<A, B>(pub A, pub B);

struct SeedPairVisitor<A, B>(A, B);

impl<'de, A: serde::de::DeserializeSeed<'de>, B: serde::de::DeserializeSeed<'de>>
    serde::de::Visitor<'de> for SeedPairVisitor<A, B>
{
    type Value = (A::Value, B::Value);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of length 2")
    }

    fn visit_seq<I>(self, mut seq: I) -> Result<Self::Value, I::Error>
    where
        I: serde::de::SeqAccess<'de>,
    {
        let val1 = seq
            .next_element_seed(self.0)?
            .ok_or_else(|| I::Error::invalid_length(0, &"a tuple of length 2"))?;
        let val2 = seq
            .next_element_seed(self.1)?
            .ok_or_else(|| I::Error::invalid_length(0, &"a tuple of length 2"))?;
        let mut len = 2;
        while let Some(IgnoredAny) = seq.next_element()? {
            len += 1;
        }

        if len != 2 {
            Err(I::Error::invalid_length(len, &"a tuple of length 2"))
        } else {
            Ok((val1, val2))
        }
    }
}

impl<'de, A: serde::de::DeserializeSeed<'de>, B: serde::de::DeserializeSeed<'de>>
    serde::de::DeserializeSeed<'de> for SeededPair<A, B>
{
    type Value = (A::Value, B::Value);
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, SeedPairVisitor(self.0, self.1))
    }
}
