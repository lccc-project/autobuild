use core::borrow::Borrow;
use core::hash::Hash;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub trait SplitOnceOwned: Sized {
    fn split_once_take(&mut self, pat: &str) -> Option<Self>;
    fn split_once_owned(mut self, pat: &str) -> Result<(Self, Self), Self> {
        match self.split_once_take(pat) {
            Some(val) => Ok((self, val)),
            None => Err(self),
        }
    }
}

impl SplitOnceOwned for String {
    fn split_once_take(&mut self, pat: &str) -> Option<Self> {
        if let Some(pos) = self.find(pat) {
            let mut new_str = Vec::new();

            let off = pos + pat.len();

            new_str.extend_from_slice(&self.as_bytes()[off..]);

            self.truncate(pos);

            Some(unsafe { String::from_utf8_unchecked(new_str) })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FormatString {
    pub args: Vec<FormatArg>,
    pub rest: String,
}

impl core::fmt::Display for FormatString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for arg in &self.args {
            arg.fmt(f)?;
        }
        f.write_str(&self.rest)
    }
}

fn write_str<W: core::fmt::Write>(output: &mut W, str: &str) -> std::io::Result<()> {
    output
        .write_str(str)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "fmt error"))
}

impl FormatString {
    pub fn eval<W: core::fmt::Write, S: Borrow<str> + Eq + Hash, R: AsRef<str>>(
        &self,
        default: &str,
        keys: &HashMap<S, R>,
        mut output: W,
    ) -> std::io::Result<()> {
        for arg in &self.args {
            write_str(&mut output, &arg.leading_text)?;
            match &arg.fmt {
                FormatSpec::EscapeLeftBrace => write_str(&mut output, "{")?,
                FormatSpec::EscapeRightBrace => write_str(&mut output, "}")?,
                FormatSpec::Default => write_str(&mut output, default)?,
                FormatSpec::Keyed(key) => {
                    let val = keys.get(&**key).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unknown key {key}"),
                        )
                    })?;

                    write_str(&mut output, val.as_ref())?;
                }
            }
        }

        write_str(&mut output, &self.rest)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FormatArg {
    pub leading_text: String,
    pub fmt: FormatSpec,
}

impl core::fmt::Display for FormatArg {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(&self.leading_text)?;
        self.fmt.fmt(f)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FormatSpec {
    EscapeLeftBrace,  // {{
    EscapeRightBrace, // }}
    Default,          // {}
    Keyed(String),    // {*str*}
}

impl core::fmt::Display for FormatSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EscapeLeftBrace => f.write_str("{{"),
            Self::EscapeRightBrace => f.write_str("}}"),
            Self::Default => f.write_str("{}"),
            Self::Keyed(key) => f.write_fmt(format_args!("{{{}}}", key)),
        }
    }
}

fn parse_fmt_str<'a, E: serde::de::Error>(x: &'a str) -> Result<(Option<FormatArg>, &'a str), E> {
    if let Some((l, r)) = x.split_once('{') {
        if r.starts_with('{') {
            let r = &r[1..];
            Ok((
                Some(FormatArg {
                    leading_text: l.to_string(),
                    fmt: FormatSpec::EscapeLeftBrace,
                }),
                r,
            ))
        } else if let Some((arg, r)) = r.split_once('}') {
            if arg.is_empty() {
                Ok((
                    Some(FormatArg {
                        leading_text: l.to_string(),
                        fmt: FormatSpec::Default,
                    }),
                    r,
                ))
            } else {
                Ok((
                    Some(FormatArg {
                        leading_text: l.to_string(),
                        fmt: FormatSpec::Keyed(arg.to_string()),
                    }),
                    r,
                ))
            }
        } else {
            Err(E::invalid_value(
                serde::de::Unexpected::Str(x),
                &FormatStringVisitor,
            ))
        }
    } else if let Some((l, r)) = x.split_once('}') {
        if r.starts_with('}') {
            let r = &r[1..];
            Ok((
                Some(FormatArg {
                    leading_text: l.to_string(),
                    fmt: FormatSpec::EscapeRightBrace,
                }),
                r,
            ))
        } else {
            Err(E::invalid_value(
                serde::de::Unexpected::Str(x),
                &FormatStringVisitor,
            ))
        }
    } else {
        Ok((None, x))
    }
}

struct FormatStringVisitor;

impl<'de> serde::de::Visitor<'de> for FormatStringVisitor {
    type Value = FormatString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a format string containing plain text, format keys like {} or {foo}, and escaped {{ and }} sequences (interpreted as literal { and } respectively)")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_str<E>(self, mut v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut args = Vec::new();

        loop {
            let (arg, rest) = parse_fmt_str(v)?;
            v = rest;
            match arg {
                Some(arg) => args.push(arg),
                None => break,
            }
        }

        Ok(FormatString {
            args,
            rest: v.to_string(),
        })
    }
}

impl<'de> serde::Deserialize<'de> for FormatString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FormatStringVisitor)
    }
}

impl serde::Serialize for FormatString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let st = self.to_string();

        serializer.serialize_str(&st)
    }
}

pub fn which<S: AsRef<Path> + ?Sized>(prg: &S) -> io::Result<PathBuf> {
    let path = std::env::var_os("PATH")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "PATH not set"))?;

    for mut file in std::env::split_paths(&path) {
        file.push(prg);
        if !std::env::consts::EXE_EXTENSION.is_empty() {
            file.set_extension(std::env::consts::EXE_EXTENSION);
        }

        if let Ok(perms) = std::fs::metadata(&file) {
            if crate::os::is_executable(&perms.permissions()) {
                return Ok(file);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Program {} not found in PATH", prg.as_ref().display()),
    ))
}
