use std::marker::PhantomData;

use serde::de::{Error, Unexpected};
use target_tuples::Target;

use crate::map::OrderedMap;

use super::ConfigTargets;

pub struct TargetVisitor;

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

impl<'de> serde::de::Deserialize<'de> for ConfigTargets {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        enum ConfigTargetsFieldIndex {
            Build,
            Host,
            Target,
        }
        enum ConfigTargetsField {
            Named(ConfigTargetsFieldIndex),
            Other(String),
        }

        impl ConfigTargetsFieldIndex {
            pub fn as_str(&self) -> &str {
                match self {
                    ConfigTargetsFieldIndex::Build => "build",
                    ConfigTargetsFieldIndex::Host => "host",
                    ConfigTargetsFieldIndex::Target => "target",
                }
            }
        }

        struct ConfigTargetsFieldVisitor;

        impl<'de> serde::de::Visitor<'de> for ConfigTargetsFieldVisitor {
            type Value = ConfigTargetsField;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`build`, `host`, or `target")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "build" => Ok(ConfigTargetsField::Named(ConfigTargetsFieldIndex::Build)),
                    "host" => Ok(ConfigTargetsField::Named(ConfigTargetsFieldIndex::Host)),
                    "target" => Ok(ConfigTargetsField::Named(ConfigTargetsFieldIndex::Target)),
                    x => Ok(ConfigTargetsField::Other(x.to_string())),
                }
            }
        }

        impl<'de> serde::de::Deserialize<'de> for ConfigTargetsField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(ConfigTargetsFieldVisitor)
            }
        }

        pub struct ConfigTargetsVisitor;

        impl<'de> serde::de::Visitor<'de> for ConfigTargetsVisitor {
            type Value = ConfigTargets;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("target name cache")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut targ = [None, None, None];
                let mut others = OrderedMap::new();
                while let Some((key, val)) =
                    map.next_entry_seed(PhantomData::<ConfigTargetsField>, DeserializeTarget)?
                {
                    match key {
                        ConfigTargetsField::Other(key) => {
                            if let Some(_) = others.insert(key, val) {
                                return Err(A::Error::custom(format_args!("Duplicate key")));
                            }
                        }
                        ConfigTargetsField::Named(field) => {
                            if let Some(_) = targ[field as usize].replace(val) {
                                return Err(A::Error::custom(format_args!(
                                    "Duplicate key `{}`",
                                    field.as_str()
                                )));
                            }
                        }
                    }
                }

                let [build, host, target] = targ;

                let build = build.ok_or_else(|| A::Error::custom("missing field `build`"))?;
                let host = host.ok_or_else(|| A::Error::custom("missing field `host`"))?;
                let target = target.ok_or_else(|| A::Error::custom("missing field `target`"))?;

                Ok(ConfigTargets {
                    build,
                    host,
                    target,
                    others,
                })
            }
        }

        deserializer.deserialize_map(ConfigTargetsVisitor)
    }
}

impl serde::ser::Serialize for ConfigTargets {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut serializer = serializer.serialize_map(Some(3 + self.others.len()))?;

        serializer.serialize_entry("build", self.build.get_name())?;
        serializer.serialize_entry("host", self.host.get_name())?;
        serializer.serialize_entry("target", self.host.get_name())?;

        for (key, val) in &self.others {
            serializer.serialize_entry(key, val.get_name())?;
        }

        serializer.end()
    }
}
