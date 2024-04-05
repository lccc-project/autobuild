use std::{collections::HashSet, ffi::OsStr, io, path::Path, process::Command, slice::Iter};

use serde_derive::{Deserialize, Serialize};

use crate::config::Config;

use target_tuples::{Architecture, Target};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RustcTarget {
    pub real_target: String,
    pub rustc_target: String,
    pub rlib_prefix: String,
    pub rlib_suffix: String,
    pub dylib_prefix: String,
    pub dylib_suffix: String,
    pub staticlib_prefix: String,
    pub staticlib_suffix: String,
    pub cdylib_prefix: String,
    pub cdylib_suffix: String,
    pub bin_prefix: String,
    pub bin_suffix: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RustcCli {
    Rustc,
    Gcc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RustcVersion {
    pub cli: RustcCli,
    pub target: RustcTarget,
    pub supported_editions: HashSet<RustEdition>,
    pub features_available: HashSet<RustcFeature>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RustEdition {
    Rust2015,
    Rust2018,
    Rust2021,
    Rust2024,
}

impl RustEdition {
    pub fn rustc_edition_year(&self) -> &'static str {
        match self {
            Self::Rust2015 => "2015",
            Self::Rust2018 => "2018",
            Self::Rust2021 => "2021",
            Self::Rust2024 => "2024",
        }
    }

    // pub fn gcc_edition_name(&self) -> &'static str {
    //     match self {
    //         Self::Rust2015 => "rust2015",
    //         Self::Rust2018 => "rust2018",
    //         Self::Rust2021 => "rust2021",
    //         Self::Rust2024 => "rust2024",
    //     }
    // }

    pub fn all() -> RustEditions {
        RustEditions(
            [
                Self::Rust2015,
                Self::Rust2018,
                Self::Rust2021,
                Self::Rust2024,
            ]
            .iter(),
        )
    }
}

pub struct RustEditions(Iter<'static, RustEdition>);

impl Iterator for RustEditions {
    type Item = RustEdition;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RustcFeature {
    AllowNightly,
    LCRustV0,
}

fn test_target_rustc<P: AsRef<OsStr>>(
    rustc: &P,
    tempfile: &Path,
    actual_target: &mut String,
    try_target: String,
) -> io::Result<Option<RustcTarget>> {
    let rustc = rustc.as_ref();

    let rustc_lossy = rustc.to_string_lossy();
    // This logic (likely) is wrong for `rustc`, but who makes x86_64-pc-linux-gnu-rustc be rustc anyways
    let cmd: std::process::Output = if rustc_lossy.contains(&try_target) {
        Command::new(rustc)
            .arg("--crate-name")
            .arg("__")
            .arg("--crate-type")
            .arg("rlib,dylib,staticlib,cdylib,bin")
            .arg("--print")
            .arg("file-names")
            .arg(tempfile)
            .output()?
    } else {
        Command::new(rustc)
            .arg("--target")
            .arg(&try_target)
            .arg("--crate-name")
            .arg("__")
            .arg("--crate-type")
            .arg("rlib,dylib,staticlib,cdylib,bin")
            .arg("--print")
            .arg("file-names")
            .arg(tempfile)
            .output()?
    };

    if cmd.status.success() {
        let file_names = String::from_utf8(cmd.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file_names = file_names.lines();
        let rlib_name = file_names
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Expected a file name"))?;
        let dylib_name = file_names
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Expected a file name"))?;
        let staticlib_name = file_names
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Expected a file name"))?;
        let cdylib_name = file_names
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Expected a file name"))?;
        let bin_name = file_names
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Expected a file name"))?;

        let (rlib_prefix, rlib_suffix) = rlib_name
            .split_once("__")
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap();
        let (dylib_prefix, dylib_suffix) = dylib_name
            .split_once("__")
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap();
        let (staticlib_prefix, staticlib_suffix) = staticlib_name
            .split_once("__")
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap();
        let (cdylib_prefix, cdylib_suffix) = cdylib_name
            .split_once("__")
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap();
        let (bin_prefix, bin_suffix) = bin_name
            .split_once("__")
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap();

        Ok(Some(RustcTarget {
            real_target: core::mem::take(actual_target),
            rustc_target: try_target,
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
        }))
    } else {
        Ok(None)
    }
}

fn test_rustc_cli<P: AsRef<OsStr>>(_: &P, _: &Path) -> io::Result<RustcCli> {
    // Assume it's rustc for now, figure out a heuristic for gcc later
    Ok(RustcCli::Rustc)
}

pub fn rustc_detect_target<P: AsRef<OsStr>>(
    temp_file: &Path,
    rustc: &P,
    mut target: String,
) -> io::Result<RustcTarget> {
    let try_target = target.clone();

    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }
    let parsed = Target::parse(&target);
    let try_target = parsed.to_string();
    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }

    let try_target = format!("{}-{}", parsed.arch_name(), parsed.sys());
    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }
    let try_target = {
        use core::fmt::Write as _;
        let mut st = String::new();
        let _ = writeln!(st, "{}", parsed.arch());
        if let Some(os) = parsed.operating_system() {
            let _ = writeln!(st, "-{}", os);
        }
        let mut has_sep = false;
        if let Some(env) = parsed.environment() {
            has_sep = true;
            let _ = writeln!(st, "-{}", env);
        }

        if let Some(obj) = parsed.object_format() {
            if !has_sep {
                st.push('-');
            }
            let _ = writeln!(st, "{}", obj);
        }
        st
    };
    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }

    let try_target = format!("{}-unknown-{}", parsed.arch_name(), parsed.sys());
    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }
    let try_target = {
        use core::fmt::Write as _;
        let mut st = String::new();
        let _ = writeln!(st, "{}-unknown", parsed.arch());
        if let Some(os) = parsed.operating_system() {
            let _ = writeln!(st, "-{}", os);
        }
        let mut has_sep = false;
        if let Some(env) = parsed.environment() {
            has_sep = true;
            let _ = writeln!(st, "-{}", env);
        }

        if let Some(obj) = parsed.object_format() {
            if !has_sep {
                st.push('-');
            }
            let _ = writeln!(st, "{}", obj);
        }
        st
    };

    if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
        return Ok(targ);
    }

    if parsed.arch().is_x86() || parsed.arch() == Architecture::X86_64 {
        let try_target = format!("{}-pc-{}", parsed.arch_name(), parsed.sys());
        if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
            return Ok(targ);
        }
        let try_target = {
            use core::fmt::Write as _;
            let mut st = String::new();
            let _ = writeln!(st, "{}-pc", parsed.arch());
            if let Some(os) = parsed.operating_system() {
                let _ = writeln!(st, "-{}", os);
            }
            let mut has_sep = false;
            if let Some(env) = parsed.environment() {
                has_sep = true;
                let _ = writeln!(st, "-{}", env);
            }

            if let Some(obj) = parsed.object_format() {
                if !has_sep {
                    st.push('-');
                }
                let _ = writeln!(st, "{}", obj);
            }
            st
        };

        if let Some(targ) = test_target_rustc(rustc, temp_file, &mut target, try_target)? {
            return Ok(targ);
        }
    }

    // TODO: Add more heuristics

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Could not determine the target for {} (real target {})",
            rustc.as_ref().to_string_lossy(),
            target
        ),
    ))
}

fn rustc_test_edition<P: AsRef<OsStr>>(
    temp_file: &Path,
    rustc: &P,
    edition: RustEdition,
) -> io::Result<bool> {
    let output = Command::new(rustc)
        .args([
            "--crate-name",
            "__",
            "--crate-type",
            "rlib",
            "--print",
            "crate-name",
            "--edition",
            edition.rustc_edition_year(),
        ])
        .arg(temp_file)
        .output()?;

    Ok(output.status.success())
}

pub fn rustc_info<P: AsRef<OsStr>>(
    temp_file: &Path,
    rustc: &P,
    target: String,
) -> io::Result<RustcVersion> {
    let cli = test_rustc_cli(rustc, &temp_file)?;

    match cli {
        RustcCli::Rustc => {
            let target = rustc_detect_target(temp_file, rustc, target)?;

            let supported_editions = RustEdition::all()
                .filter_map(
                    |edition| match rustc_test_edition(temp_file, rustc, edition) {
                        Ok(true) => Some(Ok(edition)),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    },
                )
                .collect::<io::Result<HashSet<_>>>()?;

            let features_available = HashSet::new();

            Ok(RustcVersion {
                cli,
                target,
                supported_editions,
                features_available,
            })
        }
        RustcCli::Gcc => todo!("gcc-style CLIs"),
    }
}

pub fn info<P: AsRef<OsStr>>(
    cfg: &mut Config,
    rustc: &P,
    target: String,
) -> io::Result<RustcVersion> {
    let tempfile = cfg.temp_file("rs")?;
    rustc_info(&tempfile, rustc, target)
}
