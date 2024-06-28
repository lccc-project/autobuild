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

macro_rules! impl_serde{
    {
        $tyname:ident = $defaults:ident {
            $($field:ident $(as $name:literal)?),*
            $(,)?
        }
    } => {
        const _: () = {
            use ::serde::ser::SerializeStruct as _;
            const __FIELD_COUNT: usize = (0 $(+(1,::core::stringify!($field)).0)*);

            const __FIELDS: [&'static str; __FIELD_COUNT] = [$(($($name,)?::core::stringify!($field),).0),*];

            impl ::serde::ser::Serialize for $tyname{
                fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok,__S::Error> where __S: ::serde::ser::Serializer{
                    let mut fields = serializer.serialize_struct(::core::stringify!($tyname),__FIELD_COUNT)?;

                    $(fields.serialize_field(($($name,)?::core::stringify!($field),).0, &self.$field)?;)*

                    fields.end()
                }
            }

            #[allow(non_camel_case_types)]
            enum __Field{
                $($field),*
            }

            struct __FieldVisitor;

            impl<'de> ::serde::de::Visitor<'de> for __FieldVisitor{
                type Value = __Field;

                #[allow(unused_mut,unused_variables, unused_assignments)]
                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result{
                    formatter.write_str("one of: ")?;
                    let mut sep = "";

                    $({
                        formatter.write_str(sep)?;
                        sep = ", ";
                        formatter.write_str(($($name,)?::core::stringify!($field),).0)?;
                    })*

                    Ok(())
                }

                fn visit_str<__E>(self, value: &str) -> Result<__Field,__E> where __E: ::serde::de::Error{
                    match value{
                        $($($name|)? ::core::stringify!($field) => Ok(__Field:: $field),)*
                        value => Err(::serde::de::Error::unknown_field(value, &__FIELDS))
                    }
                }
            }

            impl<'de> ::serde::de::Deserialize<'de> for __Field{
                fn deserialize<__D>(deserializer: __D) -> Result<__Field,__D::Error> where __D: ::serde::de::Deserializer<'de>{
                    deserializer.deserialize_identifier(__FieldVisitor)
                }
            }

            struct __Visitor;

            impl<'de> ::serde::de::Visitor<'de> for __Visitor{
                type Value = $tyname;

                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result{
                    formatter.write_str(::core::concat!("struct ", ::core::stringify!($tyname)))
                }

                fn visit_seq<__V>(self, mut seq: __V) -> Result<$tyname, __V::Error> where __V: ::serde::de::SeqAccess<'de>{
                    let mut __length = 0;
                    $(let $field = seq.next_element()?.ok_or_else(|| ::serde::de::Error::invalid_length({let __val = __length; __length += 1; __val},&self))?;)*

                    Ok($tyname { $($field),*})
                }

                fn visit_map<__V>(self, mut map: __V) -> Result<$tyname, __V::Error> where __V: ::serde::de::MapAccess<'de>{
                    $(let mut $field = None;)*

                    while let Some(key) = map.next_key()? {
                        match key{
                            $(__Field :: $field => {
                                if $field.is_some(){
                                    return Err(::serde::de::Error::duplicate_field(::core::stringify!($field)));
                                }

                                $field = Some(map.next_value()?);
                            })*
                        }
                    }

                    let defaults = $tyname::$defaults();

                    $(
                        let $field = $field.unwrap_or(defaults.$field);
                    )*

                    Ok($tyname {
                        $($field),*
                    })
                }
            }

            impl<'de> ::serde::de::Deserialize<'de> for $tyname{
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error> where __D: ::serde::de::Deserializer<'de>{
                    deserializer.deserialize_struct(::core::stringify!($tyname), &__FIELDS, __Visitor)
                }
            }
        };

    }
}

pub(crate) use impl_serde;
