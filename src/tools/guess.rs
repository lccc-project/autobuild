use std::env::Args;
use std::io;

pub fn main(prg_name: &str, args: Args) -> io::Result<()> {
    for opt in args {
        match &*opt {
            "--help" => {
                super::print_help(prg_name, "guess", || println!());
                return Ok(());
            }
            "--version" => {
                super::print_version();
                return Ok(());
            }
            x => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Option: {} is not recognized", x),
                ))
            }
        }
    }

    let targ = crate::targ::guess::ident_target();
    println!("{}", targ);
    Ok(())
}
