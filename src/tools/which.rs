use std::env::Args;
use std::io;

use crate::helpers;

fn help_subcommands() {
    println!("which [OPTIONS] filename...");
    println!("Finds programs on PATH which can be executed on shell");
    println!("Options:");
    println!("\t--help: Prints this message and exits");
    println!("\t--version: Prints version information and exits")
}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    while let Some(name) = args.next() {
        match &*name {
            "--help" => {
                super::print_help(prg_name, "which", help_subcommands);
                return Ok(());
            }
            "--version" => {
                super::print_version();
                return Ok(());
            }
            x => {
                let path = helpers::which(x)?;

                println!("{}", path.display());
            }
        }
    }

    Ok(())
}
