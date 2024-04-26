fn gen_os_string() -> Option<&'static str> {
    let mut os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if let Ok(env) = std::env::var("CARGO_CFG_TARGET_ENV") {
        os += "-";
        os += &env;
    }

    match &*os {
        "linux-gnu" => Some("GNU/Linux"),
        "linux-musl" => Some("MUSL/Linux"),
        "linux-uclibc" => Some("uClibc/Linux"),
        "linux-android" => Some("Android"),
        "none" => None,
        "uefi" => Some("UEFI"),
        "windows-gnu" | "mingw32" | "mingw64" => Some("MinGW"),
        // Note: rustc's windows-gnullvm target may behave differently than lccc's windows-gnu target.
        // Explore relevant differences as applied to host compilation with lccc, and adjust target name/guess support accordingly
        // Do not merge this line of code with the above one
        #[allow(clippy::match_same_arms)] // Separated identical arms for rationale above
        "windows-gnullvm" => Some("MinGW"),
        "cygwin" => Some("Cygwin"),
        _ => None,
    }
}

fn main() {
    let env = std::env::var("CARGO_PKG_VERSION").unwrap();

    if let Some(st) = gen_os_string() {
        println!("cargo:rustc-env=HOST_OS_NAME={}", st);
    }

    println!("cargo:rustc-env=VERSION={}", env);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/targ/guess.data");
}
