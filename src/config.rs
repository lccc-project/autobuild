use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::env::VarError;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::{path::PathBuf, str::FromStr};

use io::Read as _;

use install_dirs::dirs::InstallDirs;
use serde::de::Visitor;
use serde_derive::{Deserialize, Serialize};

use target_tuples::Target;

use crate::hash::sha::Sha64State;
use crate::hash::{self, FileHash};
use crate::helpers::{which, FormatString};
use crate::map::OrderedMap;
use crate::programs::rustc;
use crate::rand::Rand;

pub mod script;

mod store;

#[derive(Clone, Debug)]
pub enum ConfigVarValue {
    Set,
    Unset,
    Value(String),
}

impl serde::Serialize for ConfigVarValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ConfigVarValue::Set => serializer.serialize_bool(true),
            ConfigVarValue::Unset => serializer.serialize_none(),
            ConfigVarValue::Value(v) => serializer.serialize_str(v),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ConfigVarValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = ConfigVarValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a boolean")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigVarValue::Unset)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigVarValue::Unset)
            }

            fn visit_some<D: serde::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                deserializer.deserialize_any(self)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v {
                    Ok(ConfigVarValue::Set)
                } else {
                    Ok(ConfigVarValue::Unset)
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigVarValue::Value(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigVarValue::Value(v.to_string()))
            }
        }

        deserializer.deserialize_option(ValueVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigInstallDirs {
    #[serde(flatten)]
    pub install_dirs: InstallDirs,
    #[serde(flatten)]
    pub rest: OrderedMap<String, PathBuf>,
}

#[derive(Clone, Debug)]
pub struct ConfigTargets {
    pub build: Target,
    pub host: Target,
    pub target: Target,
    pub others: OrderedMap<String, Target>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFoundProgram {
    pub location: PathBuf,
    #[serde(flatten)]
    pub info: Option<ConfigProgramInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConfigProgramInfo {
    Rustc(rustc::RustcVersion),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TargetNameFromStrError;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TargetName {
    pub base_path: PathBuf,
    pub name: String,
}

impl core::fmt::Display for TargetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_path.display().fmt(f)?;
        f.write_str(":")?;
        f.write_str(&self.name)
    }
}

impl core::str::FromStr for TargetName {
    type Err = TargetNameFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((l, r)) = s.split_once(":") {
            Ok(Self {
                base_path: PathBuf::from(l),
                name: r.to_string(),
            })
        } else {
            Err(TargetNameFromStrError)
        }
    }
}

impl serde::ser::Serialize for TargetName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let st = self.to_string();

        serializer.serialize_str(&st)
    }
}

impl<'de> serde::de::Deserialize<'de> for TargetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TargetNameVisitor;
        impl<'de> Visitor<'de> for TargetNameVisitor {
            type Value = TargetName;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string of the form path:target")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse()
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(TargetNameVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub deps: Vec<PathBuf>,
    pub target: TargetName,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubdirInfo {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildInfo {
    pub compiler_name: String,
    pub primary_artifacts: Vec<PathBuf>,
    pub secondary_artifacts: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum BuildTargetStep {
    Empty,
    Subdir(SubdirInfo),
    Build(BuildInfo),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildTargetInfo {
    pub deps: Vec<TargetName>,
    #[serde(flatten)]
    pub step: BuildTargetStep,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubdirCache {
    #[serde(flatten)]
    vars: OrderedMap<String, ConfigVarValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigData {
    pub serial: FileHash,
    pub src_dir: PathBuf,
    pub dirs: ConfigInstallDirs,
    pub env: OrderedMap<String, String>,
    pub programs: OrderedMap<String, ConfigFoundProgram>,
    pub targets: ConfigTargets,
    pub file_cache: OrderedMap<String, FileHash>,
    pub config_vars: OrderedMap<String, ConfigVarValue>,
    pub global_key: FileHash,
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub build_database: OrderedMap<TargetName, BuildTargetInfo>,
    #[serde(default)]
    pub cache_vars: OrderedMap<PathBuf, SubdirCache>,
}

impl ConfigData {
    pub fn new(
        src_dir: PathBuf,
        dirs: ConfigInstallDirs,
        targets: ConfigTargets,
        rand: &mut Rand,
    ) -> Self {
        Self {
            serial: FileHash::ZERO,
            src_dir,
            dirs,
            env: OrderedMap::new(),
            programs: OrderedMap::new(),
            targets,
            file_cache: OrderedMap::new(),
            config_vars: OrderedMap::new(),
            global_key: FileHash::generate_key(rand),
            artifacts: Vec::new(),
            build_database: OrderedMap::new(),
            cache_vars: OrderedMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ExtraConfigDirs {
    #[serde(flatten)]
    pub dirs: OrderedMap<String, FormatString>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ProgramSpec {
    #[serde(rename = "type")]
    pub ty: Option<ProgramType>,
    pub names: Vec<String>,
    pub target: Option<FormatString>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProgramType {
    Rustc,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct BuildTargets {
    #[serde(default)]
    pub groups: OrderedMap<String, GroupSpec>,
    #[serde(flatten)]
    pub targets: OrderedMap<String, TargetSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupSpec {
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    deps: Vec<String>,
    members: Vec<String>,
    #[serde(default)]
    #[serde(rename = "path-spec")]
    path_spec: Option<FormatString>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct TargetSpec {
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    deps: Vec<String>,
    #[serde(flatten)]
    step: StepSpec,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StepSpec {
    Subdir(SubdirSpec),
    Build(BuildSpec),
    Script(BuildScriptSpec),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubdirSpec {
    subdir: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum DefaultBuildType {
    Rust,
    RustProcMacro,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuildTypeInfo<'a> {
    pub allow_dependants: bool,
    pub build_compiler_name: Cow<'a, str>,
    pub compiler_name: Cow<'a, str>,
    pub compile_flags: Cow<'a, [Cow<'a, str>]>,
    pub link_flags: Cow<'a, [Cow<'a, str>]>,
    pub preprocess_flags: Cow<'a, [Cow<'a, str>]>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum BuildType {
    Default(DefaultBuildType),
    Custom(TargetName),
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct CustomBuildType {
    pub base: Option<BuildType>,
    pub allow_dependants: Option<bool>,
    pub build_compiler_name: Option<String>,
    pub compiler_name: Option<String>,
    pub compile_flags: Option<Vec<String>>,
    pub link_flags: Option<Vec<String>>,
    pub preprocess_flags: Option<Vec<String>>,
    pub deps_step: BuildStepInfo,
    pub preprocess_step: BuildStepInfo,
    pub compile_step: BuildStepInfo,
    pub link_step: BuildStepInfo,
    pub defaults: BuildInfoDefaults,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct BuildStepInfo {
    pub add_deps: AddDepsInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct AddDepsInfo {
    pub all: Vec<LibraryType>,
    #[serde(flatten)]
    pub by_type: OrderedMap<BuildType, Vec<LibraryType>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryType {
    System,
    Static,
    Dynamic,
    RlibStatic,
    RlibDynamic,
    Rlib,
    RlibProcMacro,
    DynamicFramework,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrControl {
    String(FormatString),
    Control(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct BuildArtifactInfo {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub artifact_name: Option<FormatString>,
    pub aliases: Option<Vec<FormatString>>,
    pub clean: Option<bool>,
    pub install: Option<StringOrControl>,
    pub install_link: Option<StringOrControl>,
    pub install_aliases: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct BuildLibraryInfo {
    pub library_type: Option<LibraryType>,
    pub generate_link: Option<bool>,
    pub soname: Option<StringOrControl>,
    pub soname_alias: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct BuildBinaryInfo {
    pub set_local_rpath: Option<bool>,
    pub run_target: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct BuildInfoDefaults {
    pub library: BuildLibraryInfo,
    pub binary: BuildBinaryInfo,
    pub artifact: BuildArtifactInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildOutput {
    Library(BuildLibraryInfo),
    Binary(BuildBinaryInfo),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildSpec {
    pub src: PathBuf,
    #[serde(default, rename = "type")]
    pub ty: Option<BuildType>,
    #[serde(default, flatten)]
    pub output: Option<BuildOutput>,
    #[serde(default)]
    pub artifact: BuildArtifactInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildScriptSrc {
    Build(PathBuf),
    Configure(PathBuf),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildScriptSpec {
    #[serde(flatten)]
    pub src: BuildScriptSrc,
    #[serde(default, rename = "type")]
    pub ty: Option<BuildType>,
    #[serde(default)]
    pub artifact: BuildArtifactInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Manifest {
    pub dirs: ExtraConfigDirs,
    pub programs: OrderedMap<String, ProgramSpec>,
    pub target: BuildTargets,
    pub env: Vec<String>,
}

use std::io;

#[derive(Clone, Debug)]
pub struct Config {
    data: Box<ConfigData>,
    manifests: OrderedMap<PathBuf, Manifest>,
    updated: HashSet<String>,
    cfg_dir: PathBuf,
    dirty: bool,
    rand: Rand,
    temp_dir: Option<PathBuf>,
    transient_vars: OrderedMap<PathBuf, SubdirCache>,
}

impl Config {
    pub fn new(cfg_dir: PathBuf, data: Box<ConfigData>) -> Self {
        Self {
            data,
            manifests: OrderedMap::new(),
            updated: HashSet::new(),
            dirty: true,
            cfg_dir,
            rand: Rand::init(),
            temp_dir: None,
            transient_vars: OrderedMap::new(),
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.cfg_dir
    }

    pub fn open(cfg_dir: PathBuf) -> io::Result<Self> {
        let mut cfg_path = cfg_dir.clone();
        cfg_path.push(".config.toml");
        let mut file = File::open(cfg_path)?;
        let mut st = String::new();
        file.read_to_string(&mut st)?;
        let data = Box::new(
            toml::from_str::<ConfigData>(&st)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );

        Ok(Self {
            data,
            manifests: OrderedMap::new(),
            updated: HashSet::new(),
            dirty: false,
            cfg_dir,
            rand: Rand::init(),
            temp_dir: None,
            transient_vars: OrderedMap::new(),
        })
    }

    pub fn cleanup(mut self) -> io::Result<()> {
        use io::Write;

        if self.dirty {
            self.data.serial = FileHash::generate_key(&mut self.rand);
            let mut cfg_path = self.cfg_dir.clone();
            cfg_path.push(".config.toml");
            let string = toml::to_string(self.data()).unwrap();
            let mut file = File::create(cfg_path)?;
            file.write_all(string.as_bytes())?;
        }
        if let Some(temp_dir) = &self.temp_dir {
            std::fs::remove_dir_all(temp_dir)?;
        }
        Ok(())
    }

    pub fn temp_file<S: AsRef<OsStr> + ?Sized>(&mut self, suffix: &S) -> io::Result<PathBuf> {
        let tempdir = match &mut self.temp_dir {
            Some(dir) => &*dir,
            block @ None => {
                let mut tempdir = self.cfg_dir.clone();
                tempdir.push(".temp");
                std::fs::create_dir_all(&tempdir)?;
                *block = Some(tempdir);

                block.as_ref().unwrap()
            }
        };

        let mut tempfile = tempdir.clone();

        let key = self.rand.gen();
        tempfile.push(format!("tmp{:016X}", key));
        tempfile.set_extension(suffix);

        std::fs::write(&tempfile, [])?;

        Ok(tempfile)
    }

    pub fn data(&self) -> &ConfigData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ConfigData {
        self.dirty = true;
        &mut self.data
    }

    fn check_up_to_date_with_hash(&mut self, file: String, key: FileHash) -> bool {
        if let Some(cached) = self.data().file_cache.get(&file) {
            if key == *cached {
                true
            } else {
                self.data_mut().file_cache.insert(file.clone(), key);
                self.updated.insert(file);
                false
            }
        } else {
            self.data_mut().file_cache.insert(file.clone(), key);
            self.updated.insert(file);
            false
        }
    }

    // pub fn check_up_to_date(&mut self, file: String) -> io::Result<bool> {
    //     if self.updated.contains(&file) {
    //         return Ok(false);
    //     }

    //     let mut buf = self.data().src_dir.clone();
    //     buf.push(&file);

    //     let key = match crate::hash::hash_file(&buf, Sha64State::SHA512_256, self.data().global_key)
    //     {
    //         Ok(o) => o,
    //         Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
    //         Err(e) => return Err(e),
    //     };

    //     Ok(self.check_up_to_date_with_hash(file, key))
    // }

    pub fn find_program(&mut self, key: &str, prg_spec: &ProgramSpec) -> io::Result<()> {
        if !self.data().programs.contains_key(key) {
            // If we've set a variable containing the name of the program (including via env), try to search it
            let path = if let Some(ConfigVarValue::Value(val)) = self.data().config_vars.get(key) {
                if val.contains(std::path::MAIN_SEPARATOR) {
                    if val.starts_with(std::path::MAIN_SEPARATOR) {
                        PathBuf::from(val)
                    } else {
                        // treat this as a relative path
                        std::fs::canonicalize(val)?
                    }
                } else {
                    // treat this as a program name

                    which(val)?
                }
            } else {
                let mut path = None;

                for name in &prg_spec.names {
                    println!("Checking for {}", name);
                    // treat all of these as program names

                    if let Ok(p) = which(name) {
                        println!("\tFound {}", p.display());
                        path = Some(p);
                        break;
                    }
                }

                if path.is_none() {
                    match prg_spec.ty {
                        Some(ProgramType::Rustc) => path = which("rustc").ok(),
                        _ => {}
                    }
                }

                if path.is_none() {
                    path = which(key).ok();
                }

                path.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Program {}, was not found", key),
                    )
                })?
            };

            let target = match &prg_spec.target {
                Some(fmt) => {
                    let mut keys = HashMap::new();
                    keys.insert("build", self.data().targets.build.get_name());
                    keys.insert("host", self.data().targets.host.get_name());
                    keys.insert("target", self.data().targets.target.get_name());
                    let def = self.data().targets.host.get_name();
                    let mut st = String::new();
                    fmt.eval(def, &keys, &mut st)?;
                    st
                }
                None => self.data().targets.host.get_name().to_string(),
            };

            let info = match prg_spec.ty {
                Some(ProgramType::Rustc) => {
                    Some(ConfigProgramInfo::Rustc(rustc::info(self, &path, target)?))
                }
                None => None,
            };

            let prg = ConfigFoundProgram {
                location: path,
                info,
            };

            self.data_mut().programs.insert(key.to_string(), prg);
        }

        Ok(())
    }

    pub fn get_cache_var(&self, path: &Path, key: &str) -> ConfigVarValue {
        if let Some(val) = self.data().config_vars.get(key) {
            val.clone()
        } else if let Some(val) = self
            .transient_vars
            .get(path)
            .and_then(|subdir| subdir.vars.get(key))
        {
            val.clone()
        } else if let Some(val) = self
            .data()
            .cache_vars
            .get(path)
            .and_then(|subdir| subdir.vars.get(key))
        {
            val.clone()
        } else {
            ConfigVarValue::Unset
        }
    }

    pub fn read_manifest(&mut self, src_dir: Option<PathBuf>) -> io::Result<()> {
        if let Some(src_dir) = src_dir {
            if self.manifests.contains_key(&src_dir) {
                return Ok(());
            }

            let mut manifest_file = src_dir.clone();
            manifest_file.push("autobuild.toml");

            let file = File::open(&manifest_file).map_err(|e| {
                io::Error::new(e.kind(), format!("{}: {}", manifest_file.display(), e))
            })?;

            let mut reader = hash::HashingReader::new(Sha64State::SHA512_256, file);
            reader.init(self.data().global_key);

            let mut st = String::new();

            reader.read_to_string(&mut st)?;

            let manifest = toml::from_str::<Manifest>(&st)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            let src_file_dirty = !self.check_up_to_date_with_hash(
                manifest_file.into_os_string().into_string().unwrap(),
                reader.finish(),
            );

            if src_file_dirty {
                println!("Configuring in {}", src_dir.display());

                for var in &manifest.env {
                    if !self.data().config_vars.contains_key(var) {
                        match std::env::var(var){
                            Ok(val) => {
                                self.data_mut().config_vars.insert(var.clone(), ConfigVarValue::Value(val));
                            }
                            Err(VarError::NotPresent) => {}
                            Err(VarError::NotUnicode(_)) => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Environment Variable \"{}\" was found, but contained invalid UTF-8", var)))
                        }
                    }
                }

                for (key, prg) in &manifest.programs {
                    self.find_program(key, prg)?;
                }
            }

            let rel_path = src_dir.strip_prefix(&self.data().src_dir).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Current src_dir ({}) is outside of the src_dir base",
                        src_dir.display()
                    ),
                )
            })?;

            for (name, spec) in &manifest.target.targets {
                let target_name = TargetName {
                    base_path: rel_path.to_path_buf(),
                    name: name.clone(),
                };
                let step = match &spec.step {
                    StepSpec::Subdir(subdir) => {
                        let mut subdir_path = src_dir.clone();
                        subdir_path.push(&subdir.subdir);
                        self.read_manifest(Some(subdir_path))?;
                        BuildTargetStep::Subdir(SubdirInfo {})
                    }
                    StepSpec::Build(build) => todo!(),
                    StepSpec::Script(_) => todo!(),
                };
                if src_file_dirty {
                    let deps = spec
                        .deps
                        .iter()
                        .map(|name| {
                            if name.contains(':') {
                                TargetName::from_str(name).unwrap()
                            } else {
                                TargetName {
                                    base_path: rel_path.to_path_buf(),
                                    name: name.clone(),
                                }
                            }
                        })
                        .collect();
                    self.data_mut()
                        .build_database
                        .insert(target_name, BuildTargetInfo { deps, step });
                }
            }

            for (name, group) in &manifest.target.groups {
                let target_name = TargetName {
                    base_path: rel_path.to_path_buf(),
                    name: name.clone(),
                };

                let mut group_members = vec![];

                for member in &group.members {
                    let member_target_name = format!("{}.{}", name, member);

                    let target_name = TargetName {
                        base_path: rel_path.to_path_buf(),
                        name: member_target_name,
                    };

                    let keys = HashMap::<&'static str, &'static str>::new();

                    let subdir = if let Some(spec) = &group.path_spec {
                        let mut st = String::new();
                        spec.eval(member, &keys, &mut st)?;
                        PathBuf::from(st)
                    } else {
                        PathBuf::from(member)
                    };

                    let mut subdir_path = src_dir.clone();
                    subdir_path.push(subdir);
                    self.read_manifest(Some(subdir_path))?;
                    let step = BuildTargetStep::Subdir(SubdirInfo {});

                    if src_file_dirty {
                        let deps = group
                            .deps
                            .iter()
                            .map(|name| {
                                if name.contains(':') {
                                    TargetName::from_str(name).unwrap()
                                } else {
                                    TargetName {
                                        base_path: rel_path.to_path_buf(),
                                        name: name.clone(),
                                    }
                                }
                            })
                            .collect();
                        self.data_mut()
                            .build_database
                            .insert(target_name.clone(), BuildTargetInfo { deps, step });
                    }

                    group_members.push(target_name);
                }

                if src_file_dirty {
                    self.data_mut().build_database.insert(
                        target_name,
                        BuildTargetInfo {
                            deps: group_members,
                            step: BuildTargetStep::Empty,
                        },
                    );
                }
            }

            self.manifests.insert(src_dir, manifest);

            Ok(())
        } else {
            let src_dir = self.data().src_dir.clone();
            self.read_manifest(Some(src_dir))
        }
    }
}
