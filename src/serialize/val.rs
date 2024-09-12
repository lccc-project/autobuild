use std::{borrow::Cow, marker::PhantomData};

use serde::{
    de::value::{MapAccessDeserializer, SeqAccessDeserializer},
    Deserialize, Serialize,
};

pub type Table<K, V> = crate::map::OrderedMap<K, V>;

#[derive(Clone, Debug)]
pub enum Value {
    Table(Table<String, Value>),
    List(Vec<Value>),
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Null,
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Table(val) => val.serialize(serializer),
            Value::List(val) => val.serialize(serializer),
            Value::String(val) => val.serialize(serializer),
            Value::Bool(val) => val.serialize(serializer),
            Value::Integer(val) => val.serialize(serializer),
            Value::Float(val) => val.serialize(serializer),
            Value::Null => serializer.serialize_none(),
        }
    }
}

pub mod de_owned;
pub mod ser_owned;

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                Table::deserialize(MapAccessDeserializer::new(map)).map(Value::Table)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v))
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(v))
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(val) = i64::try_from(v) {
                    self.visit_i64(val)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Other("out of range int"),
                        &"an integer in range for i64",
                    ))
                }
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Vec::deserialize(SeqAccessDeserializer::new(seq)).map(Value::List)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Clone, Debug)]
pub enum BorrowedVal<'a> {
    Table(Table<Cow<'a, str>, BorrowedVal<'a>>),
    List(Vec<BorrowedVal<'a>>),
    String(Cow<'a, str>),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Null,
}

impl<'a> BorrowedVal<'a> {
    #[allow(dead_code)]
    pub fn to_owned(&self) -> Value {
        match self {
            BorrowedVal::Table(val) => Value::Table(
                val.iter()
                    .map(|(s, val)| (s.to_string(), val.to_owned()))
                    .collect(),
            ),
            BorrowedVal::List(val) => Value::List(val.iter().map(|val| val.to_owned()).collect()),
            BorrowedVal::String(s) => Value::String(s.to_string()),
            BorrowedVal::Bool(val) => Value::Bool(*val),
            BorrowedVal::Integer(val) => Value::Integer(*val),
            BorrowedVal::Float(val) => Value::Float(*val),
            BorrowedVal::Null => Value::Null,
        }
    }
}

impl<'a> Serialize for BorrowedVal<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BorrowedVal::Table(val) => val.serialize(serializer),
            BorrowedVal::List(val) => val.serialize(serializer),
            BorrowedVal::String(val) => val.serialize(serializer),
            BorrowedVal::Bool(val) => val.serialize(serializer),
            BorrowedVal::Integer(val) => val.serialize(serializer),
            BorrowedVal::Float(val) => val.serialize(serializer),
            BorrowedVal::Null => serializer.serialize_none(),
        }
    }
}

impl<'a, 'de> Deserialize<'de> for BorrowedVal<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct ValueVisitor<'a>(PhantomData<&'a str>);

        impl<'a, 'de> serde::de::Visitor<'de> for ValueVisitor<'a>
        where
            'de: 'a,
        {
            type Value = BorrowedVal<'a>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                Table::deserialize(MapAccessDeserializer::new(map)).map(BorrowedVal::Table)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::String(Cow::Borrowed(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::String(Cow::Owned(v.to_string())))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::String(Cow::Owned(v)))
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::Integer(v))
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(val) = i64::try_from(v) {
                    self.visit_i64(val)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Other("out of range int"),
                        &"an integer in range for i64",
                    ))
                }
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::Float(v))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Vec::deserialize(SeqAccessDeserializer::new(seq)).map(BorrowedVal::List)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BorrowedVal::Null)
            }
        }

        deserializer.deserialize_any(ValueVisitor(PhantomData))
    }
}
