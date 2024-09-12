use crate::map::OrderedMap;

use super::Value;

use serde::de::Error;
use serde::de::Unexpected;
use serde::ser;

pub use serde::de::value::Error as ValueError;
use serde::ser::SerializeTupleStruct;

pub struct Serializer;

pub struct SerializeSeq(Vec<Value>);

impl ser::SerializeSeq for SerializeSeq {
    type Error = ValueError;
    type Ok = Value;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let val = value.serialize(Serializer)?;
        self.0.push(val);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Error = ValueError;
    type Ok = Value;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let val = value.serialize(Serializer)?;
        self.0.push(val);

        Ok(())
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Error = ValueError;
    type Ok = Value;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let val = value.serialize(Serializer)?;
        self.0.push(val);

        Ok(())
    }
}

struct SerializeString;

impl ser::Serializer for SerializeString {
    type Ok = String;
    type Error = ValueError;

    type SerializeSeq = ser::Impossible<String, ValueError>;

    type SerializeTuple = ser::Impossible<String, ValueError>;

    type SerializeTupleStruct = ser::Impossible<String, ValueError>;

    type SerializeTupleVariant = ser::Impossible<String, ValueError>;

    type SerializeMap = ser::Impossible<String, ValueError>;

    type SerializeStruct = ser::Impossible<String, ValueError>;

    type SerializeStructVariant = ser::Impossible<String, ValueError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Bool(v), &"a string"))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Signed(v as i64),
            &"a string",
        ))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Signed(v as i64),
            &"a string",
        ))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Signed(v as i64),
            &"a string",
        ))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Signed(v), &"a string"))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Unsigned(v as u64),
            &"a string",
        ))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Unsigned(v as u64),
            &"a string",
        ))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Unsigned(v as u64),
            &"a string",
        ))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Unsigned(v),
            &"a string",
        ))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Float(v as f64),
            &"a string",
        ))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Float(v), &"a string"))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode_utf8(&mut [0; 4]).to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Bytes(v), &"a string"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Other("none"),
            &"a string",
        ))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Unit, &"a string"))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(name.to_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        Err(ValueError::invalid_type(Unexpected::Seq, &"a string"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Seq, &"a string"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Seq, &"a string"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Seq, &"a string"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::TupleVariant,
            &"a string",
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ValueError::invalid_type(Unexpected::Unit, &"a string"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Other("struct"),
            &"a string",
        ))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ValueError::invalid_type(
            Unexpected::Other("struct variant"),
            &"a string",
        ))
    }
}

pub struct SerializeMap(OrderedMap<String, Value>, Option<String>);

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ser::Serialize,
        V: ser::Serialize,
    {
        let st = key.serialize(SerializeString)?;

        let val = value.serialize(Serializer)?;

        self.0.insert(st, val);

        Ok(())
    }

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let key = key.serialize(SerializeString)?;

        if let Some(outstanding) = self.1.replace(key) {
            Err(ValueError::custom(format_args!(
                "unconsumed key `{}` outstanding",
                outstanding
            )))
        } else {
            Ok(())
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        if let Some(key) = self.1.take() {
            let val = value.serialize(Serializer)?;
            self.0.insert(key, val);

            Ok(())
        } else {
            panic!("Expected a call to `serialize_key` before `serialize_value`")
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(outstanding) = self.1 {
            Err(ValueError::custom(format_args!(
                "unconsumed key `{}` outstanding",
                outstanding
            )))
        } else {
            Ok(Value::Table(self.0))
        }
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let val = value.serialize(Serializer)?;

        self.0.insert(key.to_string(), val);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Table(self.0))
    }
}

pub struct SerializeVariant<V>(String, V);

impl ser::SerializeTupleVariant for SerializeVariant<SerializeSeq> {
    type Error = ValueError;
    type Ok = Value;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        self.1.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let val = self.1.end()?;

        Ok(Value::Table(core::iter::once((self.0, val)).collect()))
    }
}

impl ser::SerializeStructVariant for SerializeVariant<SerializeMap> {
    type Error = ValueError;
    type Ok = Value;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        use ser::SerializeStruct as _;
        self.1.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        use ser::SerializeStruct as _;
        let val = self.1.end()?;

        Ok(Value::Table(core::iter::once((self.0, val)).collect()))
    }
}

impl ser::Serializer for Serializer {
    type Ok = Value;

    type Error = ValueError;

    type SerializeSeq = SerializeSeq;

    type SerializeTuple = SerializeSeq;

    type SerializeTupleStruct = SerializeSeq;

    type SerializeTupleVariant = SerializeVariant<SerializeSeq>;

    type SerializeMap = SerializeMap;

    type SerializeStruct = SerializeMap;

    type SerializeStructVariant = SerializeVariant<SerializeMap>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        v.try_into()
            .map_err(|_| {
                ValueError::invalid_value(
                    Unexpected::Other("out of range i128"),
                    &"an integer in range for i64",
                )
            })
            .map(Value::Integer)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        v.try_into()
            .map_err(|_| {
                ValueError::invalid_value(Unexpected::Unsigned(v), &"an integer in range for i64")
            })
            .map(Value::Integer)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        v.try_into()
            .map_err(|_| {
                ValueError::invalid_value(
                    Unexpected::Other("out of range u128"),
                    &"an integer in range for i64",
                )
            })
            .map(Value::Integer)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.collect_seq(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(name.to_string()))
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let value = value.serialize(self)?;

        Ok(Value::Table(
            core::iter::once((variant.to_string(), value)).collect(),
        ))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len.unwrap_or(0))))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeVariant(
            variant.to_string(),
            SerializeSeq(Vec::with_capacity(len)),
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap(
            OrderedMap::with_capacity(len.unwrap_or(0)),
            None,
        ))
    }

    fn serialize_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeMap(OrderedMap::with_capacity(len), None))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeVariant(
            variant.to_string(),
            SerializeMap(OrderedMap::with_capacity(len), None),
        ))
    }

    fn is_human_readable(&self) -> bool {
        true // `FileHash` hashes to a hex string on human readable formats - this is more efficient than a List of bytes
    }
}
