use crate::targ::guess::uname::uname;

use std::{collections::HashSet, env::Args, io};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum PrintType {
    Kernel,
    Hostname,
    Krelease,
    Kver,
    Mach,
    Sys,
}

fn help() {}

pub fn main(prg_name: &str, args: Args) -> io::Result<()> {
    let uname = uname()?;

    let mut print_types = HashSet::new();

    for arg in args {
        match &*arg {
            "--help" => {
                super::print_help(prg_name, "uname", help);
                return Ok(());
            }
            "--version" => {
                super::print_version();
                return Ok(());
            }
            "--all" => print_types.extend([
                PrintType::Kernel,
                PrintType::Hostname,
                PrintType::Krelease,
                PrintType::Kver,
                PrintType::Mach,
                PrintType::Sys,
            ]),
            "--kernel-name" => {
                let _ = print_types.insert(PrintType::Kernel);
            }
            "--nodename" => {
                let _ = print_types.insert(PrintType::Hostname);
            }
            "--kernel-release" => {
                let _ = print_types.insert(PrintType::Krelease);
            }
            "--kernel-version" => {
                let _ = print_types.insert(PrintType::Kver);
            }
            "--machine" => {
                let _ = print_types.insert(PrintType::Mach);
            }
            "--operating-system" => {
                let _ = print_types.insert(PrintType::Sys);
            }
            x if x.starts_with("--") => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Argument {} not recognized", x),
                ));
            }

            x if x.starts_with("-") => {
                for c in x[1..].chars() {
                    match c {
                        'a' => print_types.extend([
                            PrintType::Kernel,
                            PrintType::Hostname,
                            PrintType::Krelease,
                            PrintType::Kver,
                            PrintType::Mach,
                            PrintType::Sys,
                        ]),
                        's' => {
                            let _ = print_types.insert(PrintType::Kernel);
                        }
                        'n' => {
                            let _ = print_types.insert(PrintType::Hostname);
                        }
                        'r' => {
                            let _ = print_types.insert(PrintType::Krelease);
                        }
                        'v' => {
                            let _ = print_types.insert(PrintType::Kver);
                        }
                        'm' => {
                            let _ = print_types.insert(PrintType::Mach);
                        }
                        'o' => {
                            let _ = print_types.insert(PrintType::Sys);
                        }
                        c => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("Option -{} not recognized", c),
                            ))
                        }
                    }
                }
            }
            x => {
                super::print_help(prg_name, "uname", help);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Argument {} not recognized", x),
                ));
            }
        }
    }

    if print_types.is_empty() {
        print_types.insert(PrintType::Kernel);
    }

    let mut sep = "";
    for ty in [
        PrintType::Kernel,
        PrintType::Hostname,
        PrintType::Krelease,
        PrintType::Kver,
        PrintType::Mach,
        PrintType::Sys,
    ] {
        if print_types.contains(&ty) {
            match ty {
                PrintType::Kernel => print!("{}{}", sep, uname.kernel),
                PrintType::Hostname => print!("{}{}", sep, uname.hostname),
                PrintType::Krelease => print!("{}{}", sep, uname.krelease),
                PrintType::Kver => print!("{}{}", sep, uname.kver),
                PrintType::Mach => print!("{}{}", sep, uname.arch),
                PrintType::Sys => {
                    if let Some(sys) = &uname.sys {
                        print!("{}{}", sep, sys);
                    }
                }
            }
        }
        sep = " ";
    }
    println!();
    Ok(())
}
