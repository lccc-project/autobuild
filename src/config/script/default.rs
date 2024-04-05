use super::{BuildScriptProvider, BuildScriptTask};

use std::{collections::HashMap, ffi::OsString, io, path::Path, process::Command};

pub struct BuildScriptProviderDefault {}

impl BuildScriptProvider for BuildScriptProviderDefault {
    fn new_task<'a>(
        &'a self,
        compile_step: &'a dyn crate::programs::CompileTask,
        _: super::BuildScriptTaskTiming,
    ) -> Box<dyn super::BuildScriptTask + 'a> {
        let exec = compile_step.run_outputs()[0];

        Box::new(BuildScriptTaskDefault {
            cmd: exec,
            env: HashMap::new(),
        })
    }
}

pub struct BuildScriptTaskDefault<'a> {
    cmd: &'a Path,
    env: HashMap<String, OsString>,
}

impl<'a> BuildScriptTask for BuildScriptTaskDefault<'a> {
    fn set_autobuild_config_dir(&mut self, dir: &Path) {
        let path = dir.as_os_str().to_os_string();

        self.env.insert("AUTOBUILD_CONFIG_DIR".to_string(), path);
    }

    fn set_tempdir(&mut self, dir: &Path) {
        let path = dir.as_os_str().to_os_string();

        self.env.insert("AUTOBUILD_TEMP_DIR".to_string(), path);
    }

    fn set_cross_compiling(&mut self) {
        self.env
            .insert("CROSSCOMPILING".to_string(), OsString::from("1"));
    }

    fn set_target(&mut self, targ: &str, target: &target_tuples::Target) {
        let mut var_name = targ.to_ascii_uppercase();
        if !var_name.starts_with("AUTOBUILD") {
            self.env.insert(var_name, OsString::from(target.get_name()));
        }
    }

    fn set_var(&mut self, var_name: &str, value: &str, kind: super::VarKind) {
        if !var_name.starts_with("AUTOBUILD") {
            self.env
                .entry(var_name.to_string())
                .or_insert_with(|| OsString::from(value));
        }
    }

    fn set_program(&mut self, prg_name: &str, prg: &crate::config::ConfigFoundProgram) {
        if !prg_name.starts_with("AUTOBUILD") {
            self.env.insert(
                prg_name.to_string(),
                prg.location.as_os_str().to_os_string(),
            );
        }
    }

    fn set_install_dir(&mut self, key: &str, dir: &Path) {
        if !key.starts_with("AUTOBUILD") {
            self.env
                .insert(key.to_string(), dir.as_os_str().to_os_string());
        }
    }

    fn run(&self) -> io::Result<super::BuildScriptOutputs> {
        let mut cmd = Command::new(&self.cmd);
        cmd.envs(&self.env);
        cmd.env("AUTOBUILD", std::env::current_exe().unwrap());
        cmd.env("AUTOBUILD_VERSION", crate::consts::VERSION);

        todo!()
    }
}
