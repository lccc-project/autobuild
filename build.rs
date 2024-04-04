fn main() {
    let env = std::env::var("CARGO_PKG_VERSION").unwrap();

    println!("cargo:rustc-env=VERSION={}", env);
    println!("cargo:rerun-if-changed=build.rs");
}
