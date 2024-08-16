use std::{collections::HashSet, env::Args, io, path::PathBuf};

use crate::{programs, rand::Rand};

use super::require_arg;

pub fn print_help() {}

pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let delegate = require_arg(None, &mut args, None)?;

    let rustc = PathBuf::from(delegate);

    

    todo!()
}
