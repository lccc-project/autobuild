use std::collections::HashMap;
use std::env::Args;
use std::fs::File;
use std::io::{self, Read};

use std::path::{Path, PathBuf};

use target_tuples::Target;

use crate::config::{
    Config, ConfigData, ConfigInstallDirs, ConfigProgramInfo, ConfigTargets, ConfigVarValue,
};
use crate::helpers::SplitOnceOwned;
use crate::install::InstallDirs;
use crate::map::OrderedMap;
use crate::rand::Rand;

fn help() {
    println!()
}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let mut rand = Rand::init();
    let mut base_dir = None;
    let mut src_dir = None;
    let mut cfg_dir = None;
    let mut config_vars = OrderedMap::new();
    let mut extra_install_dirs = OrderedMap::new();

    let mut install_dirs = InstallDirs::default();
    let mut install_dirs_dirty = false;

    let mut build_alias = None;
    let mut host_alias = None;
    let mut target_alias = None;

    install_dirs.read_env();

    while let Some(mut arg) = args.next() {
        let explicit_arg = arg.split_once_take("=");
        match &*arg {
            "--help" => {
                if let Some(explicit_arg) = explicit_arg {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unrecognized option --help={}", explicit_arg),
                    ));
                }
                super::print_help(prg_name, "config", help);
                return Ok(());
            }
            "--version" => {
                if let Some(explicit_arg) = explicit_arg {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unrecognized option --version={}", explicit_arg),
                    ));
                }
                super::print_version();
                return Ok(());
            }
            "--set" => {
                let mut val = super::require_arg(Some("--set"), &mut args, explicit_arg)?;

                if let Some(v) = val.split_once_take("=") {
                    config_vars.insert(val, ConfigVarValue::Value(v));
                } else {
                    config_vars.insert(val, ConfigVarValue::Set);
                }
            }
            "--unset" => {
                let val = super::require_arg(Some("--unset"), &mut args, explicit_arg)?;

                config_vars.insert(val, ConfigVarValue::Unset);
            }
            "--install" => {
                let val = super::require_arg(Some("--install"), &mut args, explicit_arg)?;

                let (k,v) = val.split_once_owned("=")
                    .map_err(|val| io::Error::new(io::ErrorKind::InvalidInput, format!("--install requires an argument of the form dir=path, but got `{}` instead", val)))?;

                extra_install_dirs.insert(k, PathBuf::from(v));
            }
            "--src-dir" => {
                let val = super::require_arg(Some("--src-dir"), &mut args, explicit_arg)?;

                src_dir = Some(PathBuf::from(val));
            }

            "--config-dir" => {
                let val = super::require_arg(Some("--config-dir"), &mut args, explicit_arg)?;

                cfg_dir = Some(PathBuf::from(val));
            }
            "--build" => {
                let val = super::require_arg(Some("--build"), &mut args, explicit_arg)?;

                build_alias = Some(val);
            }

            "--host" => {
                let val = super::require_arg(Some("--host"), &mut args, explicit_arg)?;

                host_alias = Some(val);
            }

            "--target" => {
                let val = super::require_arg(Some("--target"), &mut args, explicit_arg)?;

                target_alias = Some(val);
            }

            "--" => {
                if let Some(explicit_arg) = explicit_arg {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unrecognized option --={}", explicit_arg),
                    ));
                }
                base_dir = args.next().map(PathBuf::from);
                break;
            }
            x if x.starts_with("--") => {
                let val = super::require_arg(Some(x), &mut args, explicit_arg)?;

                install_dirs.set_from_arg(x, val).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unrecognized option {}", x),
                    )
                })?;

                install_dirs_dirty = true;
            }

            _ => {
                base_dir = Some(PathBuf::from(arg));
                break;
            }
        }
    }

    let base_dir = base_dir.ok_or(()).or_else(|_| std::env::current_dir())?;

    let base_dir = base_dir.canonicalize()?;

    let cfg_dir = match cfg_dir {
        Some(cfg_dir) => cfg_dir.canonicalize()?,
        None => {
            let mut base_dir_config = base_dir.clone();
            base_dir_config.push(".config.toml");

            if std::fs::metadata(base_dir_config).is_ok() {
                base_dir.clone()
            } else {
                std::env::current_dir()?
            }
        }
    };

    let mut config = match Config::open(cfg_dir.clone()) {
        Ok(mut config) => {
            if install_dirs_dirty {
                let data = config.data_mut();
                data.dirs.install_dirs.set_from(&install_dirs);
            }

            for (key, dir) in extra_install_dirs {
                let data = config.data_mut();
                data.dirs.rest.insert(key, dir);
            }
            config
        }
        Err(_) => {
            let src_dir = match src_dir {
                Some(src_dir) => src_dir.canonicalize()?,
                None => {
                    let mut src_dir = base_dir;

                    loop {
                        let mut autobuild_file = src_dir.clone();
                        autobuild_file.push("autobuild.toml");

                        if std::fs::metadata(autobuild_file).is_ok() {
                            break;
                        } else if &src_dir == Path::new("/") {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "Could not find source manifest (autobuild.toml)",
                            ));
                        } else {
                            src_dir.pop();
                        }
                    }

                    src_dir
                }
            };

            let build = match build_alias.as_ref() {
                Some(alias) => alias.parse().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Target `{}` is not recognized or cannot be parsed", alias),
                    )
                })?,
                None => crate::targ::guess::ident_target(),
            };

            let host = match host_alias.as_ref() {
                Some(alias) => alias.parse().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Target `{}` is not recognized or cannot be parsed", alias),
                    )
                })?,
                None => build.clone(),
            };

            let target = match target_alias.as_ref() {
                Some(alias) => alias.parse().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Target `{}` is not recognized or cannot be parsed", alias),
                    )
                })?,
                None => host.clone(),
            };

            let dirs = ConfigInstallDirs {
                install_dirs,
                rest: extra_install_dirs,
            };

            let targets = ConfigTargets {
                build,
                host,
                target,
                others: OrderedMap::new(),
            };
            Config::new(
                cfg_dir.clone(),
                Box::new(ConfigData::new(src_dir, dirs, targets, &mut rand)),
            )
        }
    };

    for (key, val) in config_vars {
        let data = config.data_mut();

        data.config_vars.insert(key, val);
    }

    config.read_manifest(None)?;

    config.cleanup()
}
