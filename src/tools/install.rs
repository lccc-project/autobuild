use std::env::Args;
use std::io;
use std::path::PathBuf;

use crate::install::InstallDirs;
use crate::map::OrderedMap;

use crate::helpers::SplitOnceOwned;

fn print_help() {}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let mut config_dir = PathBuf::new();
    let mut install_dirs = InstallDirs::default();
    let mut sysroot = None;
    let mut extra_install_dirs = OrderedMap::new();
    let mut dry_run = false;
    let mut install_command = None;
    let mut strip_command = None;
    let mut strip_debug = false;
    let mut targets = Vec::new();

    while let Some(mut arg) = args.next() {
        let explicit = arg.split_once_take("=");

        match &*arg {
            "--install-dir" => {
                let arg = super::require_arg(Some("--install-dir"), &mut args, explicit)?;
                let (dir_name, value) = arg.split_once_owned("=").map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid argument for --install-dir \"{e}\". Argument must be in key=value form")))?;
                extra_install_dirs.insert(dir_name, value);
            }
            "--user-prefix" => {
                install_dirs.prefix = Some(
                    dirs::home_dir()
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                "Could not find user home directory",
                            )
                        })
                        .map(|mut p| {
                            p.push(".local");
                            p
                        })?,
                );
            }
            "--strip" => {
                if explicit.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--strip does not accept an argument",
                    ));
                }
                strip_debug = true;
            }
            "--strip-command" => {
                strip_command = Some(super::require_arg(
                    Some("--strip-command"),
                    &mut args,
                    explicit,
                )?);
            }
            "--install-command" => {
                install_command = Some(super::require_arg(
                    Some("--install-command"),
                    &mut args,
                    explicit,
                )?);
            }
            "--sysroot" => {
                sysroot = Some(PathBuf::from(super::require_arg(
                    Some("--sysroot"),
                    &mut args,
                    explicit,
                )?));
            }
            "--dry-run" => {
                if explicit.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--strip does not accept an argument",
                    ));
                }
                dry_run = true;
            }
            "--config-dir" => {
                config_dir = PathBuf::from(super::require_arg(
                    Some("--config-dir"),
                    &mut args,
                    explicit,
                )?);
            }
            "--" => break,
            x if x.starts_with("--") => {
                install_dirs
                    .set_from_arg(x, super::require_arg(Some(x), &mut args, explicit)?)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Option {} not recognized", x),
                        )
                    });
            }
            _ => {
                targets.push(arg);
                break;
            }
        }
    }

    targets.extend(args);

    Ok(())
}
