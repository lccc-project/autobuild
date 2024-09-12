use target_tuples::Target;

use crate::programs::CompileTask;
use std::io;

use super::{ConfigFoundProgram, ConfigVarValue};

use std::{collections::HashMap, path::Path};

pub mod default;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BuildScriptTaskTiming {
    Configure,
    CompileBuild,
    CompileHost,
    InstallHost,
    InstallTarget,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarKind {
    DirTransient,
    DirCache,
    Config,
}

pub trait BuildScriptProvider {
    fn new_task<'a>(
        &'a self,
        compile_step: &'a dyn CompileTask,
        timing: BuildScriptTaskTiming,
    ) -> Box<dyn BuildScriptTask + 'a>;
}

pub struct BuildScriptOutputs {
    pub set_transient: HashMap<String, ConfigVarValue>,
    pub set_cache: HashMap<String, ConfigVarValue>,
}

pub trait BuildScriptTask {
    fn set_var(&mut self, var_name: &str, value: &str, kind: VarKind);
    fn set_program(&mut self, prg_name: &str, prg: &ConfigFoundProgram);
    fn set_install_dir(&mut self, key: &str, dir: &Path);
    fn set_target(&mut self, targ: &str, target: &Target);
    fn set_cross_compiling(&mut self);
    fn set_autobuild_config_dir(&mut self, dir: &Path);
    fn set_tempdir(&mut self, dir: &Path);

    fn run(&self) -> io::Result<BuildScriptOutputs>;
}
