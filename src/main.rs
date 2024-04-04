use std::env::Args;
use std::io;

mod config;
mod consts;
mod fs;
mod hash;
mod helpers;
mod jobserv;
mod map;
mod os;
mod programs;
mod rand;
mod serialize;
mod targ;
mod tools;

fn main() {
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
