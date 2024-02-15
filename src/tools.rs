use std::env::Args;
use std::ffi::OsStr;
use std::io;

macro_rules! def_tools{
    {
        $(tool $tool:ident $(alias $($else:ident)+)?;)*
    } => {
        $(pub mod $tool;)*

        pub fn find_tool(x: &str) -> io::Result<fn(&str, Args)->io::Result<()>>{
            match x{
                $(::core::stringify!($tool) $($(| ::core::stringify!($else))+)? => Ok($tool::main),)*
                x => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("No such subcommand {}", x)))
            }
        }

        pub fn print_help(prg_name: &str, tool_name: &str, help_cb: fn()){
            println!("Usage: {} <subcommand>",prg_name);
            println!("Available Subcommands:");
            $(println!("\t{}", ::core::stringify!($tool));)*

            println!("{} {} Usage:", prg_name, tool_name);
            help_cb()
        }

        pub fn help_subcommands<T>() -> io::Result<T>{
            Err(io::Error::new(io::ErrorKind::InvalidInput, {
                use core::fmt::Write;
                let mut st = String::new();

                let _ = writeln!(st, "Subcommands:");
                $(let _ = writeln!(st, "\t{}", ::core::stringify!($tool));)*

                st
            }))
        }
    }
}

def_tools! {
    tool config alias configure;
    tool rustc;
}

pub fn print_version() {
    println!("cargo-autobuild v{}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2024 LCCC Maintainers");
    println!("This Package is released under the terms of the 2BSD License + Patent Grant");
    println!("See LICENSE for details");
}

pub fn require_arg<I: Iterator<Item = String>>(
    flag: Option<&str>,
    args: &mut I,
) -> io::Result<String> {
    args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            if let Some(flag) = flag {
                format!("{} requires an argument", flag)
            } else {
                format!("Expected an argument")
            },
        )
    })
}

pub fn invoke_tool<I: IntoIterator>(tool_name: &str, args: I) -> std::io::Result<String>
where
    I::Item: AsRef<OsStr>,
{
    let cmd = std::env::current_exe()?;

    String::from_utf8(
        std::process::Command::new(cmd)
            .arg(tool_name)
            .args(args)
            .output()?
            .stdout,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
