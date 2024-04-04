use std::{
    any::{Any, TypeId},
    ffi::OsStr,
    path::{Path, PathBuf},
};

use target_tuples::Target;

use crate::hash::FileHash;

pub mod rustc;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CompileTaskType {
    pub raw_build_type: String,
    pub leaf: bool,
    pub executable: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DepInfo {
    pub dep_path: PathBuf,
    pub hash: FileHash,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CompileTaskStep {
    Compile,
    Link,
}

pub trait Compiler {
    fn abs_path(&self) -> &Path;
    fn default_flags(&self) -> &[&OsStr];
    fn target(&self) -> &Target;
    fn create_compile_task<'a>(
        &'a self,
        output: &'a Path,
        name: &'a str,
        build_type: CompileTaskType,
    ) -> Box<dyn CompileTask + 'a>;
}

pub trait CompileTask {
    fn compiler(&self) -> &dyn Compiler;
    fn add_compile_lib(&mut self, lib: &Path);
    fn add_link_lib(&mut self, lib: &dyn CompileTask);
    fn add_preprocess_lib(&mut self, lib: &dyn CompileTask);
    fn add_compile_flag(&mut self, flag: &OsStr);
    fn add_link_flag(&mut self, flag: &OsStr);
    fn add_preprocess_flag(&mut self, flag: &OsStr);
    fn name(&self) -> &str;
    fn link_outputs(&self) -> Vec<&Path>;
    fn run_outputs(&self) -> Vec<&Path>;
    fn gather_deps(&self) -> std::io::Result<Vec<DepInfo>>;
    fn run_steps(
        &mut self,
        from: CompileTaskStep,
        to: CompileTaskStep,
    ) -> std::io::Result<Vec<DepInfo>>;
}
