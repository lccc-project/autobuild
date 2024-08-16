use std::env::Args;
use std::io::{self, IsTerminal};

mod config;
mod consts;
mod fs;
mod hash;
mod helpers;
mod install;
mod jobserv;
mod log;
mod map;
mod os;
mod programs;
mod rand;
mod serialize;
mod set;
mod targ;
mod tools;
mod cmd;

fn main() {
    let log_level = if let Ok(mut arg) = std::env::var("AUTOBUILD_LOG") {
        arg.make_ascii_lowercase();
        match &*arg {
            "0" | "off" | "no" | "n" => log::LogLevel::Off,
            "1" | "fatal" => log::LogLevel::Fatal,
            "2" | "error" => log::LogLevel::Error,
            "3" | "warn" | "warning" => log::LogLevel::Warning,
            "4" | "diagnostic" | "on" | "yes" | "y" | "" => log::LogLevel::Diagnostic,
            "5" | "exec" => log::LogLevel::Exec,
            "6" | "verbose" | "v" => log::LogLevel::Verbose,
            "7" | "info" | "i" => log::LogLevel::Info,
            "8" | "trace" | "full" => log::LogLevel::Trace,
            "9" | "debug" => log::LogLevel::Debug,
            _ => panic!(
                "AUTOBUILD_LOG set to invalid value \"{}\"",
                std::env::var("AUTOBUILD_LOG").unwrap()
            ),
        }
    } else {
        log::LogLevel::Diagnostic
    };

    let use_term_fmt = if let Ok(mut arg) = std::env::var("AUTOBUILD_TERM_COLORS") {
        arg.make_ascii_lowercase();
        match &*arg {
            "off" | "no" | "false" | "never" | "0" => false,
            "default" | "tty" | "" => std::io::stderr().is_terminal(),
            "on" | "yes" | "true" | "always" | "1" => true,
            _ => panic!(
                "AUTOBUILD_TERM_COLORS set to invalid value \"{}\"",
                std::env::var("AUTOBUILD_TERM_COLORS").unwrap()
            ),
        }
    } else {
        std::io::stderr().is_terminal()
    };

    log::set_logging_config(log_level, use_term_fmt);

    let mut args = std::env::args();

    let mut prg_name = args.next().unwrap();

    if prg_name == "cargo" {
        prg_name.push(' ');
        prg_name += &args.next().unwrap();
    }

    match real_main(&prg_name, args) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}: {}", prg_name, e);

            std::process::exit(1);
        }
    }
}

fn real_main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let subcommand = args
        .next()
        .ok_or_else(|| ())
        .or_else(|_| tools::help_subcommands())?;

    match &*subcommand {
        "--help" => {
            println!("Usage: {} <subcommand> [SUBCOMMAND ARGS...]", prg_name);
            println!("Complex build system intended for use with lccc");
            tools::print_subcommands();
            return Ok(());
        }
        "--version" => {
            tools::print_version();
            return Ok(());
        }
        _ => {}
    }

    let subcommand_entry = tools::find_tool(&subcommand)?;

    subcommand_entry(prg_name, args)
}
