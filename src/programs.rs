use std::path::{Path, PathBuf};

pub mod rustc;

pub trait Compiler {
    fn generate_deps(&self, src: &Path) -> Vec<PathBuf>;
}
