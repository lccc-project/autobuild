use crate::serialize::helpers::DeserializeTarget;

use serde::de::{Error, IgnoredAny};

use serde::ser::SerializeStruct;

use super::RustcTarget;

impl serde::Serialize for RustcTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("RustcTarget", 12)?;
        s.serialize_field("real-target", self.real_target.get_name())?;
        s.serialize_field("rustc-target", &self.rustc_target)?;
        s.serialize_field("rlib-prefix", &self.rlib_prefix)?;
        s.serialize_field("rlib-suffix", &self.rlib_suffix)?;
        s.serialize_field("dylib-prefix", &self.dylib_prefix)?;
        s.serialize_field("dylib-suffix", &self.dylib_suffix)?;
        s.serialize_field("staticlib-prefix", &self.staticlib_prefix)?;
        s.serialize_field("staticlib-suffix", &self.staticlib_suffix)?;
        s.serialize_field("cdylib-prefix", &self.cdylib_prefix)?;
        s.serialize_field("cdylib-suffix", &self.cdylib_suffix)?;
        s.serialize_field("bin-prefix", &self.bin_prefix)?;
        s.serialize_field("bin-suffix", &self.bin_suffix)?;

        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for RustcTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Copy, Clone)]
        enum RustcTargetField {
            RealTarget,
            RustcTarget,
            RlibPrefix,
            RlibSuffix,
            DylibPrefix,
            DylibSuffix,
            StaticlibPrefix,
            StaticlibSuffix,
            CdylibPrefix,
            CdylibSuffix,
            BinPrefix,
            BinSuffix,
            __Other,
        }

        impl RustcTargetField {
            fn name(&self) -> &'static str {
                match self {
                    Self::RealTarget => "real-target",
                    Self::RustcTarget => "rustc-target",
                    Self::RlibPrefix => "rlib-prefix",
                    Self::RlibSuffix => "rlib-suffix",
                    Self::DylibPrefix => "dylib-prefix",
                    Self::DylibSuffix => "dylib-suffix",
                    Self::StaticlibPrefix => "staticlib-prefix",
                    Self::StaticlibSuffix => "staticlib-suffix",
                    Self::CdylibPrefix => "cdylib-prefix",
                    Self::CdylibSuffix => "cdylib-suffix",
                    Self::BinPrefix => "bin-prefix",
                    Self::BinSuffix => "bin-suffix",
                    Self::__Other => "",
                }
            }
        }

        struct RustcTargetFieldVisitor;

        impl<'de> serde::de::Visitor<'de> for RustcTargetFieldVisitor {
            type Value = RustcTargetField;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`real-target`, `rustc-target`, `rlib-prefix`, `rlib-suffix`, `dylib-prefix`, `dylib-suffix`, `staticlib-prefix`, `staticlib-suffix`, `cdylib-prefix`, `cdylib-suffix`, `bin-prefix`, or `bin-suffix`")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "real-target" | "real_target" => Ok(RustcTargetField::RealTarget),
                    "rustc-target" | "rustc_target" => Ok(RustcTargetField::RustcTarget),
                    "rlib-prefix" | "rlib_prefix" => Ok(RustcTargetField::RlibPrefix),
                    "rlib-suffix" | "rlib_suffix" => Ok(RustcTargetField::RlibSuffix),
                    "dylib-prefix" | "dylib_prefix" => Ok(RustcTargetField::DylibPrefix),
                    "dylib-suffix" | "dylib_suffix" => Ok(RustcTargetField::DylibSuffix),
                    "staticlib-prefix" | "staticlib_prefix" => {
                        Ok(RustcTargetField::StaticlibPrefix)
                    }
                    "staticlib-suffix" | "staticlib_suffix" => {
                        Ok(RustcTargetField::StaticlibSuffix)
                    }
                    "cdylib-prefix" | "cdylib_prefix" => Ok(RustcTargetField::CdylibPrefix),
                    "cdylib-suffix" | "cdylib_suffix" => Ok(RustcTargetField::CdylibSuffix),
                    "bin-prefix" | "bin_preifx" => Ok(RustcTargetField::BinPrefix),
                    "bin-suffix" | "bin_suffix" => Ok(RustcTargetField::BinSuffix),
                    _ => Ok(RustcTargetField::__Other),
                }
            }
        }

        impl<'de> serde::de::DeserializeSeed<'de> for RustcTargetFieldVisitor {
            type Value = RustcTargetField;
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(self)
            }
        }

        struct RustcTargetVisitor;

        impl<'de> serde::de::Visitor<'de> for RustcTargetVisitor {
            type Value = RustcTarget;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("information about a rustc target")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                const __VAL: Option<String> = None;
                let mut target = None;
                let mut fields = [__VAL; 11];

                while let Some(key) = map.next_key_seed(RustcTargetFieldVisitor)? {
                    match key {
                        RustcTargetField::RealTarget => {
                            if target
                                .replace(map.next_value_seed(DeserializeTarget)?)
                                .is_some()
                            {
                                return Err(A::Error::duplicate_field("real-target"));
                            }
                        }
                        RustcTargetField::__Other => continue,
                        x => {
                            let val = x as usize - 1;

                            if fields[val].replace(map.next_value::<String>()?).is_some() {
                                return Err(A::Error::duplicate_field(x.name()));
                            }
                        }
                    }
                }

                let real_target = target.ok_or_else(|| A::Error::missing_field("real-target"))?;

                let [rustc_target, rlib_prefix, rlib_suffix, dylib_prefix, dylib_suffix, staticlib_prefix, staticlib_suffix, cdylib_prefix, cdylib_suffix, bin_prefix, bin_suffix] =
                    fields;

                Ok(RustcTarget {
                    real_target,
                    rustc_target: rustc_target
                        .ok_or_else(|| A::Error::missing_field("rustc-target"))?,
                    rlib_prefix: rlib_prefix
                        .ok_or_else(|| A::Error::missing_field("rlib-prefix"))?,
                    rlib_suffix: rlib_suffix
                        .ok_or_else(|| A::Error::missing_field("rlib-suffix"))?,
                    dylib_prefix: dylib_prefix
                        .ok_or_else(|| A::Error::missing_field("dylib-prefix"))?,
                    dylib_suffix: dylib_suffix
                        .ok_or_else(|| A::Error::missing_field("dylib-suffix"))?,
                    staticlib_prefix: staticlib_prefix
                        .ok_or_else(|| A::Error::missing_field("staticlib-prefix"))?,
                    staticlib_suffix: staticlib_suffix
                        .ok_or_else(|| A::Error::missing_field("staticlib-suffix"))?,
                    cdylib_prefix: cdylib_prefix
                        .ok_or_else(|| A::Error::missing_field("cdylib-prefix"))?,
                    cdylib_suffix: cdylib_suffix
                        .ok_or_else(|| A::Error::missing_field("cdylib-suffix"))?,
                    bin_prefix: bin_prefix.ok_or_else(|| A::Error::missing_field("bin-prefix"))?,
                    bin_suffix: bin_suffix.ok_or_else(|| A::Error::missing_field("bin-suffix"))?,
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let real_target = seq
                    .next_element_seed(DeserializeTarget)?
                    .ok_or_else(|| A::Error::missing_field("real-target"))?;
                let rustc_target = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("rustc-target"))?;
                let rlib_prefix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("rlib-prefix"))?;
                let rlib_suffix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("rlib-suffix"))?;
                let dylib_prefix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("dylib-prefix"))?;
                let dylib_suffix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("dylib-suffix"))?;
                let staticlib_prefix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("staticlib-prefix"))?;
                let staticlib_suffix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("staticlib-suffix"))?;
                let cdylib_prefix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("cdylib-prefix"))?;
                let cdylib_suffix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("cdylib-suffix"))?;
                let bin_prefix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("bin-prefix"))?;
                let bin_suffix = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("bin-suffix"))?;

                while let Some(IgnoredAny) = seq.next_element()? {}

                Ok(RustcTarget {
                    real_target,
                    rustc_target,
                    rlib_prefix,
                    rlib_suffix,
                    dylib_prefix,
                    dylib_suffix,
                    staticlib_prefix,
                    staticlib_suffix,
                    cdylib_prefix,
                    cdylib_suffix,
                    bin_prefix,
                    bin_suffix,
                })
            }
        }

        deserializer.deserialize_struct(
            "RustcTarget",
            &[
                "real-target",
                "rustc-target",
                "rlib-prefix",
                "rlib-suffix",
                "dylib-prefix",
                "dylib-suffix",
                "staticlib-prefix",
                "staticlib-suffix",
                "cdylib-prefix",
                "cdylib-suffix",
                "bin-prefix",
                "bin-suffix",
            ],
            RustcTargetVisitor,
        )
    }
}
