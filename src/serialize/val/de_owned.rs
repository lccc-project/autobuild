use std::marker::PhantomData;

use serde::{
    de::{
        value::{SeqDeserializer, StringDeserializer},
        DeserializeSeed, EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess, Unexpected,
        VariantAccess,
    },
    forward_to_deserialize_any, Deserializer,
};

use crate::map::{self, OrderedMap};

use super::Value;

pub use serde::de::value::Error as ValueError;

enum ExpectedInMap {
    Count(usize),
    OutstandingValue,
}

impl serde::de::Expected for ExpectedInMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExpectedInMap::Count(val) => f.write_fmt(format_args!("a map with {} elements", val)),
            ExpectedInMap::OutstandingValue => f.write_fmt(format_args!("a key-value pair")),
        }
    }
}

struct UnitVariantAccess;

impl<'de> Deserializer<'de> for UnitVariantAccess {
    type Error = ValueError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_none()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf newtype_struct map
        enum identifier ignored_any unit unit_struct
    }
}

impl<'de> SeqAccess<'de> for UnitVariantAccess {
    type Error = ValueError;

    fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Ok(None)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(None)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(0)
    }
}

impl<'de> VariantAccess<'de> for UnitVariantAccess {
    type Error = ValueError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if len == 0 {
            visitor.visit_unit()
        } else {
            Err(ValueError::invalid_type(Unexpected::UnitVariant, &visitor))
        }
    }
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if fields.len() == 0 {
            visitor.visit_unit()
        } else {
            Err(ValueError::invalid_type(Unexpected::UnitVariant, &visitor))
        }
    }
}

struct ValueVariantAccess(Value);

impl<'de> EnumAccess<'de> for ValueVariantAccess {
    type Error = ValueError;
    type Variant = UnitVariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.0).map(|v| (v, UnitVariantAccess))
    }
}

impl<'de> VariantAccess<'de> for ValueVariantAccess {
    type Error = ValueError;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.0)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.deserialize_struct("<don't care>", fields, visitor)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.deserialize_tuple(len, visitor)
    }

    fn unit_variant(self) -> Result<(), Self::Error> {
        PhantomData.deserialize(self.0)
    }
}

struct MapDeserializer(map::IntoIter<String, Value>, Option<Value>, usize);

impl MapDeserializer {
    pub fn new(map: OrderedMap<String, Value>) -> Self {
        Self(map.into_iter(), None, 0)
    }

    pub fn end<E: serde::de::Error>(self) -> Result<(), E> {
        let remaining = self.0.count();

        if remaining != 0 {
            Err(E::invalid_length(
                self.2 + remaining,
                &ExpectedInMap::Count(self.2),
            ))
        } else if let Some(val) = self.1 {
            let unexp = val_as_unexpected(&val);
            Err(E::invalid_value(unexp, &ExpectedInMap::OutstandingValue))
        } else {
            Ok(())
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = ValueError;

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
        V: serde::de::DeserializeSeed<'de>,
    {
        let (key, val) = match self.0.next() {
            Some(entry) => entry,
            None => return Ok(None),
        };

        self.2 += 1;

        let key = kseed.deserialize(StringDeserializer::new(key))?;
        let val = vseed.deserialize(val)?;

        Ok(Some((key, val)))
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let (key, val) = match self.0.next() {
            Some(entry) => entry,
            None => return Ok(None),
        };

        self.2 += 1;

        let key = seed.deserialize(StringDeserializer::new(key))?;
        self.1 = Some(val);

        Ok(Some(key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = self
            .1
            .take()
            .expect("next_key must be called before next_value, or use next_entry instead");

        seed.deserialize(val)
    }
}

impl<'de> EnumAccess<'de> for MapDeserializer {
    type Error = ValueError;

    type Variant = ValueVariantAccess;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if let Some((key, val)) = self.0.next() {
            self.end()?;

            seed.deserialize(StringDeserializer::new(key))
                .map(|v| (v, ValueVariantAccess(val)))
        } else {
            Err(ValueError::invalid_length(0, &"a map with at one element"))
        }
    }
}

impl<'de> Deserializer<'de> for MapDeserializer {
    type Error = ValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let val = visitor.visit_map(&mut self)?;
        self.end()?;

        Ok(val)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.end()?;

        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.end()?;

        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option newtype_struct tuple seq
        tuple_struct  identifier ignored_any
    }
}

fn val_as_unexpected(val: &Value) -> Unexpected {
    match val {
        Value::Table(_) => Unexpected::Map,
        Value::List(_) => Unexpected::Seq,
        Value::String(st) => Unexpected::Str(&st),
        Value::Bool(b) => Unexpected::Bool(*b),
        Value::Integer(int) => Unexpected::Signed(*int),
        Value::Float(float) => Unexpected::Float(*float),
        Value::Null => Unexpected::Unit,
    }
}

impl<'de> IntoDeserializer<'de> for Value {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = ValueError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Table(table) => MapDeserializer::new(table).deserialize_any(visitor),
            Value::List(list) => {
                let list_access = SeqDeserializer::new(list.into_iter());

                list_access.deserialize_any(visitor)
            }
            Value::String(s) => visitor.visit_string(s),
            Value::Integer(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f64(f),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Bool(b) => visitor.visit_bool(b),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_i8(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_i16(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_i32(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => visitor.visit_i64(val),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => visitor.visit_i64(val),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_u8(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_u16(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_u32(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val.try_into().map_err(|_| {
                    ValueError::invalid_value(Unexpected::Signed(val), &"a value in range for i8")
                })?;
                visitor.visit_u64(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val as f32;
                visitor.visit_f32(val)
            }
            Value::Float(val) => {
                let val = val as f32;
                visitor.visit_f32(val)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(val) => {
                let val = val as f64;
                visitor.visit_f64(val)
            }
            Value::Float(val) => visitor.visit_f64(val),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(s) => {
                let mut chars = s.chars();
                let c1 = chars
                    .next()
                    .ok_or_else(|| ValueError::invalid_length(0, &"a string with one character"))?;
                let count = chars.count();

                if count != 0 {
                    return Err(ValueError::invalid_length(
                        count,
                        &"a string with one character",
                    ));
                }

                visitor.visit_char(c1)
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_str(&s),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_string(s),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            val => visitor.visit_some(val),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Table(_) => visitor.visit_unit(),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Table(tab) => {
                let map = MapDeserializer::new(tab);
                map.end()?;
                visitor.visit_unit()
            }
            Value::String(st) => {
                if &st == name {
                    visitor.visit_unit()
                } else {
                    Err(ValueError::invalid_value(Unexpected::Str(&st), &visitor))
                }
            }
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::List(list) => visitor.visit_seq(SeqDeserializer::new(list.into_iter())),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::List(list) => visitor.visit_seq(SeqDeserializer::new(list.into_iter())),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::List(list) => visitor.visit_seq(SeqDeserializer::new(list.into_iter())),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Table(tab) => visitor.visit_map(MapDeserializer::new(tab)),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Table(tab) => visitor.visit_map(MapDeserializer::new(tab)),
            val => Err(ValueError::invalid_type(val_as_unexpected(&val), &visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Table(tab) => visitor.visit_enum(MapDeserializer::new(tab)),
            val => visitor.visit_enum(ValueVariantAccess(val)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    forward_to_deserialize_any!(ignored_any bytes byte_buf);
}
