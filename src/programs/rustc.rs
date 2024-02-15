use std::{collections::HashSet, ffi::OsStr, io};

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RustcTarget {
    pub real_target: String,
    pub actual_target: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Cli {
    Rustc,
    Gcc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RustcVersion {
    pub supported_editions: HashSet<RustEdition>,
    pub features_available: HashSet<RustcFeature>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RustEdition {
    Rust2015,
    Rust2018,
    Rust2021,
    Rust2024,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RustcFeature {
    AllowNightly,
    LCRustV0,
}

pub fn info<P: AsRef<OsStr>>(rustc: &P, target: String) -> io::Result<RustcVersion> {
    Ok(RustcVersion {
        supported_editions: HashSet::new(),
        features_available: HashSet::new(),
    })
}
