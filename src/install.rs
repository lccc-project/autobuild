use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::log::{log_debug, trace};
mod store;

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct InstallDirs {
    pub prefix: Option<PathBuf>,
    pub exec_prefix: Option<PathBuf>,
    pub bindir: Option<PathBuf>,
    pub sbindir: Option<PathBuf>,
    pub libdir: Option<PathBuf>,
    pub libexecdir: Option<PathBuf>,
    pub includedir: Option<PathBuf>,
    pub datarootdir: Option<PathBuf>,
    pub datadir: Option<PathBuf>,
    pub mandir: Option<PathBuf>,
    pub docdir: Option<PathBuf>,
    pub infodir: Option<PathBuf>,
    pub localedir: Option<PathBuf>,
    pub localstatedir: Option<PathBuf>,
    pub runstatedir: Option<PathBuf>,
    pub sharedstatedir: Option<PathBuf>,
    pub sysconfdir: Option<PathBuf>,
}

#[allow(dead_code)]
impl InstallDirs {
    pub const fn new() -> Self {
        Self {
            prefix: None,
            exec_prefix: None,
            bindir: None,
            sbindir: None,
            libdir: None,
            libexecdir: None,
            includedir: None,
            datarootdir: None,
            datadir: None,
            mandir: None,
            docdir: None,
            infodir: None,
            localedir: None,
            localstatedir: None,
            runstatedir: None,
            sharedstatedir: None,
            sysconfdir: None,
        }
    }
    pub fn set_project_name(&mut self, name: &str) {
        trace!(InstallDirs::set_project_name);
        if cfg!(windows) {
            self.prefix
                .get_or_insert_with(|| PathBuf::from("C:\\Program Files\\"))
                .push(name);
        }
        self.docdir
            .get_or_insert_with(|| PathBuf::from("doc"))
            .push(name);
    }

    pub fn set_target(&mut self, target: &str) {
        trace!(InstallDirs::set_target);
        self.exec_prefix
            .get_or_insert_with(|| PathBuf::new())
            .push(target)
    }

    pub fn set_from_arg(&mut self, key: &str, val: String) -> Result<(), ()> {
        trace!(InstallDirs::set_from_arg);
        match key {
            "--prefix" => self.prefix = Some(PathBuf::from(val)),
            "--exec-prefix" => self.exec_prefix = Some(PathBuf::from(val)),
            "--bindir" => self.bindir = Some(PathBuf::from(val)),
            "--sbindir" => self.sbindir = Some(PathBuf::from(val)),
            "--libdir" => self.libdir = Some(PathBuf::from(val)),
            "--libexecdir" => self.libexecdir = Some(PathBuf::from(val)),
            "--includedir" => self.includedir = Some(PathBuf::from(val)),
            "--datarootdir" => self.datarootdir = Some(PathBuf::from(val)),
            "--datadir" => self.datadir = Some(PathBuf::from(val)),
            "--mandir" => self.mandir = Some(PathBuf::from(val)),
            "--docdir" => self.docdir = Some(PathBuf::from(val)),
            "--infodir" => self.infodir = Some(PathBuf::from(val)),
            "--localedir" => self.localedir = Some(PathBuf::from(val)),
            "--localstatedir" => self.localstatedir = Some(PathBuf::from(val)),
            "--runstatedir" => self.runstatedir = Some(PathBuf::from(val)),
            "--sharedstatedir" => self.sharedstatedir = Some(PathBuf::from(val)),
            "--sysconfdir" => self.sysconfdir = Some(PathBuf::from(val)),
            _ => return Err(()),
        }

        Ok(())
    }

    pub fn read_env(&mut self) {
        trace!(InstallDirs::read_env);
        if let Ok(dir) = std::env::var("prefix") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: prefix=\"{dir}\"");
            self.prefix = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("exec_prefix") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: exec_prefix=\"{dir}\""
            );
            self.exec_prefix = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("bindir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: bindir=\"{dir}\"");
            self.bindir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("libdir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: libdir=\"{dir}\"");
            self.libdir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("sbindir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: sbindir=\"{dir}\"");
            self.sbindir = Some(dir.into())
        }
        if let Ok(dir) = std::env::var("libexecdir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: libexecdir=\"{dir}\""
            );
            self.libexecdir = Some(dir.into())
        }
        if let Ok(dir) = std::env::var("includedir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: includedir=\"{dir}\""
            );
            self.includedir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("datarootdir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: datarootdir=\"{dir}\""
            );
            self.datarootdir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("datadir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: datadir=\"{dir}\"");
            self.datadir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("mandir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: mandir=\"{dir}\"");
            self.mandir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("docdir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: docdir=\"{dir}\"");
            self.docdir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("infodir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: infodir=\"{dir}\"");
            self.infodir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("localedir") {
            log_debug!(crate::log::LogLevel::Debug, "read_env: localedir=\"{dir}\"");
            self.localedir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("sharedstatedir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: sharedstatedir=\"{dir}\""
            );
            self.sharedstatedir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("localstatedir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: localstatedir=\"{dir}\""
            );
            self.localstatedir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("runstatedir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: runstatedir=\"{dir}\""
            );
            self.runstatedir = Some(dir.into())
        }

        if let Ok(dir) = std::env::var("sysconfdir") {
            log_debug!(
                crate::log::LogLevel::Debug,
                "read_env: sysconfdir=\"{dir}\""
            );
            self.sysconfdir = Some(dir.into())
        }
    }

    pub fn prefix(&self) -> &Path {
        if let Some(prefix) = self.prefix.as_ref() {
            prefix
        } else if cfg!(windows) {
            panic!("`set_project_name` must be used on windows")
        } else {
            Path::new("/usr/local")
        }
    }

    pub fn exec_prefix(&self) -> PathBuf {
        if let Some(exec_prefix) = self.exec_prefix.as_ref() {
            if exec_prefix.is_absolute() {
                exec_prefix.clone()
            } else {
                self.prefix().join(exec_prefix)
            }
        } else {
            self.prefix().to_path_buf()
        }
    }

    fn usr_escape(prefix: &Path, val: &Path) -> PathBuf {
        if val.is_absolute() {
            val.to_path_buf()
        } else if prefix.starts_with("/usr") {
            Path::new("/").join(val)
        } else {
            prefix.join(val)
        }
    }

    pub fn bindir(&self) -> PathBuf {
        let bindir = self.bindir.as_deref().unwrap_or_else(|| Path::new("bin"));

        let mut prefix = self.exec_prefix();
        prefix.push(bindir);
        prefix
    }

    pub fn libdir(&self) -> PathBuf {
        let libdir = self.libdir.as_deref().unwrap_or_else(|| Path::new("lib"));

        let mut prefix = self.exec_prefix();
        prefix.push(libdir);
        prefix
    }

    pub fn sbindir(&self) -> PathBuf {
        let sbindir = self.sbindir.as_deref().unwrap_or_else(|| Path::new("sbin"));

        let mut prefix = self.exec_prefix();
        prefix.push(sbindir);
        prefix
    }
    pub fn libexecdir(&self) -> PathBuf {
        let libexecdir = self
            .libexecdir
            .as_deref()
            .unwrap_or_else(|| Path::new("libexec"));

        let mut prefix = self.exec_prefix();
        prefix.push(libexecdir);
        prefix
    }
    pub fn includedir(&self) -> PathBuf {
        let includedir = self
            .includedir
            .as_deref()
            .unwrap_or_else(|| Path::new("include"));

        self.prefix().join(includedir)
    }

    pub fn datarootdir(&self) -> PathBuf {
        let datarootdir = self
            .datarootdir
            .as_deref()
            .unwrap_or_else(|| Path::new("share"));

        self.prefix().join(datarootdir)
    }

    pub fn datadir(&self) -> PathBuf {
        let datadir = self.datadir.as_deref().unwrap_or_else(|| Path::new("."));

        let mut prefix = self.datarootdir();
        prefix.push(datadir);
        prefix
    }

    pub fn docdir(&self) -> PathBuf {
        let docdir = self.docdir.as_deref().unwrap_or_else(|| Path::new("doc"));

        let mut prefix = self.datarootdir();
        prefix.push(docdir);
        prefix
    }

    pub fn infodir(&self) -> PathBuf {
        let infodir = self.infodir.as_deref().unwrap_or_else(|| Path::new("info"));

        let mut prefix = self.datarootdir();
        prefix.push(infodir);
        prefix
    }

    pub fn mandir(&self) -> PathBuf {
        let mandir = self.mandir.as_deref().unwrap_or_else(|| Path::new("man"));

        let mut prefix = self.datarootdir();
        prefix.push(mandir);
        prefix
    }

    pub fn localedir(&self) -> PathBuf {
        let localedir = self
            .localedir
            .as_deref()
            .unwrap_or_else(|| Path::new("locale"));

        let mut prefix = self.datarootdir();
        prefix.push(localedir);
        prefix
    }

    pub fn sysconfdir(&self) -> PathBuf {
        Self::usr_escape(
            self.prefix(),
            self.sysconfdir
                .as_deref()
                .unwrap_or_else(|| Path::new("etc")),
        )
    }

    pub fn localstatedir(&self) -> PathBuf {
        Self::usr_escape(
            self.prefix(),
            self.sysconfdir
                .as_deref()
                .unwrap_or_else(|| Path::new("var")),
        )
    }

    pub fn sharedstatedir(&self) -> PathBuf {
        let sharedstatedir = self
            .sharedstatedir
            .as_deref()
            .unwrap_or_else(|| Path::new("com"));

        self.prefix().join(sharedstatedir)
    }

    pub fn runstatedir(&self) -> PathBuf {
        let runstatedir = self
            .runstatedir
            .as_deref()
            .unwrap_or_else(|| Path::new("locale"));

        let mut prefix = self.localstatedir();
        prefix.push(runstatedir);
        prefix
    }

    /// Returns an iterator suitable for passing to [`Command::envs`][std::process::Command::envs]
    /// which contains the fully canonicalized values of each install dir.
    ///
    /// This is suitable for invoking install-time scripts, or populating a format string map.
    pub fn as_canonical_env(&self) -> impl IntoIterator<Item = (&'static str, PathBuf)> {
        [
            ("prefix", self.prefix().to_path_buf()),
            ("exec_prefix", self.exec_prefix()),
            ("bindir", self.bindir()),
            ("libdir", self.libdir()),
            ("sbindir", self.sbindir()),
            ("libexecdir", self.libexecdir()),
            ("datarootdir", self.datarootdir()),
            ("datadir", self.datadir()),
            ("docdir", self.docdir()),
            ("mandir", self.mandir()),
            ("infodir", self.infodir()),
            ("localedir", self.localedir()),
            ("sharedstatedir", self.sharedstatedir()),
            ("localstatedir", self.localstatedir()),
            ("runstatedir", self.runstatedir()),
            ("sysconfdir", self.sysconfdir()),
        ]
    }

    /// Returns an iterator suitable for passing to [`Command::envs`][std::process::Command::envs]
    /// which contains the non-canonicalized values of each install dir.
    ///
    /// This is suitable for invoking other instanstances of `autobuild`, `cargo-native-install`, configure scripts generated by autotools, or config-time scripts.
    ///
    /// Note that for invoking programs known to support the restricted install-dirs standard, [`InstallDirs::as_config_args`] should be used instead.
    ///
    /// ## Notes
    /// The current implementation will return default values for dirs that aren't explicitly set. This is not necessarily guaranteed,
    ///  and invoked programs that make use of these environment variables must accept the variables being unset.
    pub fn as_local_env(&self) -> impl IntoIterator<Item = (&'static str, &Path)> {
        [
            ("prefix", self.prefix()),
            (
                "exec_prefix",
                self.exec_prefix
                    .as_deref()
                    .unwrap_or_else(|| Path::new(".")),
            ),
            (
                "bindir",
                self.bindir.as_deref().unwrap_or_else(|| Path::new("bin")),
            ),
            (
                "libdir",
                self.libdir.as_deref().unwrap_or_else(|| Path::new("lib")),
            ),
            (
                "sbindir",
                self.sbindir.as_deref().unwrap_or_else(|| Path::new("sbin")),
            ),
            (
                "libexecdir",
                self.libexecdir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("libexec")),
            ),
            (
                "datarootdir",
                self.datarootdir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("share")),
            ),
            (
                "datadir",
                self.datadir.as_deref().unwrap_or_else(|| Path::new(".")),
            ),
            (
                "docdir",
                self.docdir.as_deref().unwrap_or_else(|| Path::new("doc")),
            ),
            (
                "mandir",
                self.mandir.as_deref().unwrap_or_else(|| Path::new("man")),
            ),
            (
                "infodir",
                self.infodir.as_deref().unwrap_or_else(|| Path::new("info")),
            ),
            (
                "localedir",
                self.localedir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("locale")),
            ),
            (
                "sharedstatedir",
                self.sharedstatedir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("com")),
            ),
            (
                "localstatedir",
                self.localstatedir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("var")),
            ),
            (
                "runstatedir",
                self.runstatedir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("run")),
            ),
            (
                "sysconfdir",
                self.sysconfdir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("etc")),
            ),
        ]
    }

    /// Returns an iterator over items suitable to pass to [`Command::args`][std::process::Command::args],
    /// which can be given to `autobuild config`, `autobuild install`, `cargo-native-install`, or a `configure` script generated by GNU autotools
    pub fn as_config_args(&self) -> impl IntoIterator<Item = &OsStr> {
        ([
            self.prefix
                .as_ref()
                .map(|v| [OsStr::new("--prefix"), v.as_os_str()]),
            self.exec_prefix
                .as_ref()
                .map(|v| [OsStr::new("--exec-prefix"), v.as_os_str()]),
            self.bindir
                .as_ref()
                .map(|v| [OsStr::new("--bindir"), v.as_os_str()]),
            self.libdir
                .as_ref()
                .map(|v| [OsStr::new("--libdir"), v.as_os_str()]),
            self.sbindir
                .as_ref()
                .map(|v| [OsStr::new("--sbindir"), v.as_os_str()]),
            self.libexecdir
                .as_ref()
                .map(|v| [OsStr::new("--libexecdir"), v.as_os_str()]),
            self.includedir
                .as_ref()
                .map(|v| [OsStr::new("--includedir"), v.as_os_str()]),
            self.datarootdir
                .as_ref()
                .map(|v| [OsStr::new("--datarootdir"), v.as_os_str()]),
            self.datadir
                .as_ref()
                .map(|v| [OsStr::new("--datadir"), v.as_os_str()]),
            self.docdir
                .as_ref()
                .map(|v| [OsStr::new("--docdir"), v.as_os_str()]),
            self.mandir
                .as_ref()
                .map(|v| [OsStr::new("--mandir"), v.as_os_str()]),
            self.infodir
                .as_ref()
                .map(|v| [OsStr::new("--infodir"), v.as_os_str()]),
            self.localedir
                .as_ref()
                .map(|v| [OsStr::new("--localedir"), v.as_os_str()]),
            self.localstatedir
                .as_ref()
                .map(|v| [OsStr::new("--localstatedir"), v.as_os_str()]),
            self.runstatedir
                .as_ref()
                .map(|v| [OsStr::new("--runstatedir"), v.as_os_str()]),
            self.sharedstatedir
                .as_ref()
                .map(|v| [OsStr::new("--sharedstatedir"), v.as_os_str()]),
            self.sysconfdir
                .as_ref()
                .map(|v| [OsStr::new("--sysconfdir"), v.as_os_str()]),
        ])
        .into_iter()
        .flatten()
        .flatten()
    }

    pub fn set_from(&mut self, other: &InstallDirs) {
        if let Some(prefix) = &other.prefix {
            self.prefix = Some(prefix.clone());
        }
        if let Some(exec_prefix) = &other.exec_prefix {
            self.exec_prefix = Some(exec_prefix.clone());
        }
        if let Some(bindir) = &other.bindir {
            self.bindir = Some(bindir.clone());
        }
        if let Some(libdir) = &other.libdir {
            self.libdir = Some(libdir.clone());
        }
        if let Some(sbindir) = &other.sbindir {
            self.sbindir = Some(sbindir.clone());
        }
        if let Some(libexecdir) = &other.libexecdir {
            self.libexecdir = Some(libexecdir.clone());
        }
        if let Some(includedir) = &other.includedir {
            self.includedir = Some(includedir.clone());
        }
        if let Some(datarootdir) = &other.datarootdir {
            self.datarootdir = Some(datarootdir.clone());
        }
        if let Some(datadir) = &other.datadir {
            self.datadir = Some(datadir.clone());
        }
        if let Some(docdir) = &other.docdir {
            self.docdir = Some(docdir.clone());
        }
        if let Some(mandir) = &other.mandir {
            self.mandir = Some(mandir.clone());
        }
        if let Some(infodir) = &other.infodir {
            self.infodir = Some(infodir.clone());
        }
        if let Some(sharedstatedir) = &other.sharedstatedir {
            self.sharedstatedir = Some(sharedstatedir.clone());
        }
        if let Some(localedir) = &other.localedir {
            self.localedir = Some(localedir.clone());
        }
        if let Some(localstatedir) = &other.localstatedir {
            self.localstatedir = Some(localstatedir.clone());
        }
        if let Some(runstatedir) = &other.runstatedir {
            self.runstatedir = Some(runstatedir.clone());
        }
        if let Some(sysconfdir) = &other.sysconfdir {
            self.sysconfdir = Some(sysconfdir.clone());
        }
    }
}
