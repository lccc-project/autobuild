use std::collections::HashSet;
use std::env::VarError;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::{collections::HashMap, convert::TryFrom, ffi::OsString, path::PathBuf, str::FromStr};

use io::Read as _;

use install_dirs::dirs::InstallDirs;
use serde_derive::{Deserialize, Serialize};

use target_tuples::{Target, UnknownError};

use crate::hash::sha::Sha64State;
use crate::hash::{self, FileHash};
use crate::helpers::{which, FormatString};
use crate::programs::rustc;
use crate::rand::Rand;

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
            ConfigVarValue::Unset => serializer.serialize_bool(false),
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

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigInstallDirs {
    #[serde(flatten)]
    pub install_dirs: InstallDirs,
    #[serde(flatten)]
    pub rest: HashMap<String, PathBuf>,
}

mod serde_target {
    use serde::{
        de::{self, Expected},
        Deserialize, Deserializer, Serializer,
    };
    use target_tuples::Target;

    pub fn serialize<S>(targ: &Target, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(targ.get_name())
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Target, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExpectedTarget;

        impl de::Expected for ExpectedTarget {
            fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expected a target in the form <arch>-<sys> or <arch>-<vendor>-<sys> with sys being one of <os>, <env>, <objfmt> or <os> followed by either <env> or <objfmt>")
            }
        }
        let ty = String::deserialize(de)?;

        ty.parse().map_err(|e| {
            <D::Error as de::Error>::invalid_value(de::Unexpected::Str(&ty), &ExpectedTarget)
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConfigTargets {
    #[serde(with = "serde_target")]
    pub build: Target,
    #[serde(with = "serde_target")]
    pub host: Target,
    #[serde(with = "serde_target")]
    pub target: Target,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub deps: Vec<PathBuf>,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigData {
    pub serial: FileHash,
    pub src_dir: PathBuf,
    pub dirs: ConfigInstallDirs,
    pub env: HashMap<String, String>,
    pub programs: HashMap<String, ConfigFoundProgram>,
    pub targets: ConfigTargets,
    pub file_cache: HashMap<String, FileHash>,
    pub config_vars: HashMap<String, ConfigVarValue>,
    pub global_key: FileHash,
    pub artifacts: Vec<Artifact>,
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
            env: HashMap::new(),
            programs: HashMap::new(),
            targets,
            file_cache: HashMap::new(),
            config_vars: HashMap::new(),
            global_key: FileHash::generate_key(rand),
            artifacts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ExtraConfigDirs {
    #[serde(flatten)]
    pub dirs: HashMap<String, FormatString>,
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
    pub groups: HashMap<String, GroupSpec>,
    #[serde(flatten)]
    pub targets: HashMap<String, TargetSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupSpec {}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct TargetSpec {
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubdirSpec {
    subdir: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildSpec {}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Manifest {
    pub dirs: ExtraConfigDirs,
    pub programs: HashMap<String, ProgramSpec>,
    pub target: BuildTargets,
    pub env: Vec<String>,
}

use std::io;

#[derive(Clone, Debug)]
pub struct Config {
    data: Box<ConfigData>,
    manifests: HashMap<PathBuf, Manifest>,
    updated: HashSet<String>,
    cfg_dir: PathBuf,
    dirty: bool,
    rand: Rand,
    temp_dir: Option<PathBuf>,
}

impl Config {
    pub fn new(cfg_dir: PathBuf, data: Box<ConfigData>) -> Self {
        Self {
            data,
            manifests: HashMap::new(),
            updated: HashSet::new(),
            dirty: true,
            cfg_dir,
            rand: Rand::init(),
            temp_dir: None,
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
            manifests: HashMap::new(),
            updated: HashSet::new(),
            dirty: false,
            cfg_dir,
            rand: Rand::init(),
            temp_dir: None,
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

    pub fn check_up_to_date(&mut self, file: String) -> io::Result<bool> {
        if self.updated.contains(&file) {
            return Ok(false);
        }

        let mut buf = self.data().src_dir.clone();
        buf.push(&file);

        let key = match crate::hash::hash_file(&buf, Sha64State::SHA512_256, self.data().global_key)
        {
            Ok(o) => o,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(e),
        };

        Ok(self.check_up_to_date_with_hash(file, key))
    }

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

    pub fn read_manifest(&mut self, src_dir: Option<PathBuf>) -> io::Result<()> {
        if let Some(src_dir) = src_dir {
            if self.manifests.contains_key(&src_dir) {
                return Ok(());
            }

            let mut manifest_file = src_dir.clone();
            manifest_file.push("autobuild.toml");

            let file = File::open(&manifest_file)?;

            let mut reader = hash::HashingReader::new(Sha64State::SHA512_256, file);

            let mut st = String::new();

            reader.read_to_string(&mut st)?;

            let manifest = toml::from_str::<Manifest>(&st)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            if !self.check_up_to_date_with_hash(
                manifest_file.into_os_string().into_string().unwrap(),
                reader.finish(),
            ) {
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

            for (name, spec) in &manifest.target.targets {}

            self.manifests.insert(src_dir, manifest);

            Ok(())
        } else {
            let src_dir = self.data().src_dir.clone();
            self.read_manifest(Some(src_dir))
        }
    }
}
