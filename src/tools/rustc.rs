use std::{collections::HashSet, env::Args, io, path::PathBuf};

use super::require_arg;

pub fn print_help() {}

pub enum RustcWrapperCli {
    Rustc(HashSet<RustcCliProperty>),
    Gcc(HashSet<GccCliProperty>),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RustcCliProperty {
    SupportsOptZ,
    SupportsOptG,
    SupportsTuneMachine,
    SupportsTuneMachineUnstable,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GccCliProperty {
    SupportsOptZ,
    SupportsOptG,
    Supports,
}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let delegate = require_arg(None, &mut args)?;

    let path = PathBuf::from(delegate);

    todo!()
}
