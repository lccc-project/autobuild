use serde::{Deserializer, Serializer};

use super::{BorrowedVal, Table, Value};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ValueError {
    Msg(String),
}

impl core::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Msg(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for ValueError {}

impl serde::de::Error for ValueError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Msg(msg.to_string())
    }
}
