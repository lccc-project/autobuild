use std::ffi::OsStr;

fn main() -> ! {
    let autobuild_path = std::env::var_os("AUTOBUILD")
        .expect("AUTOBUILD must be set in the environment to the absolute path to autobuild");
    let real_rustc_path = std::env::var_os("AUTOBUILD_CARGOEMUL_RUSTC_REAL").unwrap();
    let rest = std::env::var_os("AUTOBUILD_CARGOEMUL_RUSTC_REAL_SPECIFIER_ARGS");

    let mut args = std::env::args_os();
    args.next();

    std::process::exit(
        std::process::Command::new(autobuild_path)
            .args([OsStr::new("rustc"), &*real_rustc_path])
            .args(rest)
            .args(args)
            .status()
            .expect("Could not run process")
            .code()
            .unwrap_or(-1),
    )
}
