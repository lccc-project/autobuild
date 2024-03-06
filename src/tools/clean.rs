use std::{
    env::Args,
    fs::{self, File},
    io,
    path::PathBuf,
};

use crate::config::ConfigData;

use super::require_arg;

fn print_help() {
    println!("Cleans build directories after `autobuild config` and `autobuild build`");
    println!("\t--cache-only: Remove only the cache and ignore artifacts (same as --cache --no-artifacts)");
    println!("\t--artifacts-only: Remove only build artifacts and ignore the cache (same as --cache --no-artifacts)");
    println!("\t--cache: Remove the cache (default)");
    println!("\t--no-cache: Do not remove the cache");
    println!("\t--artifacts: Remove build artifacts (default)");
    println!("\t--no-artifacts: Do not remove build artifacts");
    println!("\t--all: Remove all files (same as --cache --artifacts)");
    println!("\t--nothing: Remove nothing (same as --no-cache --no-artifacts");
    println!("\t--verbose: Print every file removed");
    println!("\t--quiet: Print nothing (default)");
    println!("\t--fail-slow: Continue deleting as much as possible after an error");
    println!("\t--fail-fast: Stop after an error (default)");
    println!("\t--error-missing: Error about missing files and a missing cache");
    println!("\t--ignore-missing: Ignore missing files (including the cache)");
    println!("\t--error-missing=cache: When removing artifacts, error if the cache is missing or invalid (default)");
    println!("\t--ignore-missing=cache: When removing artifacts, silently ignore a missing or invalid cache");
    println!("\t--error-missing=files: When removing a file, error if it cannot be removed because it is missing");
    println!("\t--ignore-missing=files: When removing a file, ignore it if it cannot be removed because it is missing (default)");
    println!("\t--dry-run: Acknowledge files being removed, but do not try to remove them (only useful with --verbose)");
    println!("\t--clean: Remove targetted files (default)");
    println!("\t--version: Print version information and exit");
    println!("\t--help: Print this message and exit");
}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let mut clean_cache = true;
    let mut clean_artifacts = true;
    let mut verbose = false;
    let mut ignore_missing_cache = true;
    let mut ignore_missing_files = true;
    let mut fail_fast = true;
    let mut config_dir = PathBuf::new();
    let mut remove_files = true;

    while let Some(arg) = args.next() {
        match &*arg {
            "--cache-only" => {
                clean_cache = true;
                clean_artifacts = false;
            }
            "--artifacts-only" => {
                clean_cache = false;
                clean_artifacts = true;
            }
            "--cache" => {
                clean_cache = true;
            }
            "--no-cache" => {
                clean_cache = false;
            }
            "--artifacts" => {
                clean_artifacts = true;
            }
            "--no-artifacts" => {
                clean_artifacts = false;
            }
            "--verbose" => verbose = true,
            "--terse" => verbose = false,
            "--fail-slow" => fail_fast = false,
            "--fail-fast" => fail_fast = true,
            "--ignore-missing" => {
                ignore_missing_cache = true;
                ignore_missing_files = true;
            }
            "--error-missing" => {
                ignore_missing_cache = false;
                ignore_missing_files = false
            }
            "--ignore-missing=cache" => {
                ignore_missing_cache = true;
            }
            "--error-missing=cache" => {
                ignore_missing_cache = false;
            }
            "--ignore-missing=files" => {
                ignore_missing_files = true;
            }
            "--error-missing=files" => {
                ignore_missing_files = false;
            }
            "--all" => {
                clean_artifacts = true;
                clean_cache = true;
            }
            "--none" => {
                clean_artifacts = true;
                clean_cache = true;
            }
            "--cfg-dir" => {
                let dir = require_arg(Some("--cfg-dir"), &mut args, None)?;
                config_dir = PathBuf::from(dir);
            }
            "--dry-run" => {
                remove_files = false;
            }
            "--clean" => {
                remove_files = true;
            }
            "--version" => {
                super::print_version();
                return Ok(());
            }
            "--help" => {
                super::print_help(prg_name, "clean", print_help);
                return Ok(());
            }
            _ => super::help_subcommands()?,
        }
    }

    let mut res = Ok(());
    if clean_artifacts {
        let mut config_path = config_dir.clone();
        config_path.push(".config.toml");

        let open_config_res: Result<ConfigData, io::Error> = (move || {
            use io::Read as _;
            let mut file = File::open(&config_path)?;
            let mut st = String::new();
            file.read_to_string(&mut st)?;
            toml::from_str(&st).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Could not open {}: {}", config_path.display(), e),
                )
            })
        })();

        match open_config_res {
            Ok(config) => {
                let mut dirs = Vec::new();
                for art in config.artifacts {
                    let mut path: PathBuf = config_dir.clone();
                    path.push(&art.path);
                    if verbose {
                        println!("Deleting {}", art.path.display());
                    }
                    if remove_files {
                        match fs::remove_file(&path) {
                            Ok(_) => {}
                            Err(e) => {
                                match fs::metadata(&path) {
                                    Ok(m) if m.is_dir() => {
                                        dirs.push(path);
                                        continue;
                                    }
                                    _ => {}
                                }
                                if !ignore_missing_files {
                                    if fail_fast {
                                        return Err(io::Error::new(
                                            e.kind(),
                                            format!(
                                                "Could not delete {}: {}",
                                                art.path.display(),
                                                e
                                            ),
                                        ));
                                    }
                                    res = Err(io::Error::new(io::ErrorKind::Other, "failed"));
                                    eprintln!("Could not delete {}: {}", art.path.display(), e)
                                }
                            }
                        }
                    }
                }

                for dir in dirs {
                    if remove_files {
                        let _ = fs::remove_dir_all(dir);
                    }
                }
            }
            Err(e) => {
                if !ignore_missing_cache {
                    if fail_fast {
                        return res;
                    }
                    res = Err(io::Error::new(io::ErrorKind::Other, "failed"));
                    eprintln!("{}", e)
                }
            }
        }
    }

    if clean_cache {
        let mut file = config_dir.clone();
        file.push(".config.toml");
        if verbose {
            println!("Removing {}", file.display());
        }
        if remove_files {
            match fs::remove_file(&file) {
                Ok(()) => {}
                Err(e) => {
                    if !ignore_missing_files {
                        if fail_fast {
                            return Err(io::Error::new(
                                e.kind(),
                                format!("Could not delete {}: {}", file.display(), e),
                            ));
                        }
                        res = Err(io::Error::new(io::ErrorKind::Other, "failed"));
                        eprintln!("Could not delete {}: {}", file.display(), e)
                    }
                }
            }
        }
    }

    res
}
