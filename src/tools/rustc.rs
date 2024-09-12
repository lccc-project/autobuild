use std::{env::Args, io, path::PathBuf};

use super::require_arg;

#[allow(unused_variables)]
pub fn main(prg_name: &str, mut args: Args) -> io::Result<()> {
    let delegate = require_arg(None, &mut args, None)?;

    let rustc = PathBuf::from(delegate);

    todo!()
}
