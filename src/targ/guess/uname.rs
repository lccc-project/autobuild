use std::io;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Uname {
    pub kernel: String,
    pub arch: String,
    pub sys: Option<String>,
}

pub fn uname() -> io::Result<Uname> {
    cfg_match::cfg_match! {
        unix => ({
            let kernel = std::process::Command::new("uname").arg("-s").output()?;
            let arch = std::process::Command::new("uname").arg("-m").output()?;
            let sys = std::process::Command::new("uname").arg("-o").output()?;

            if !kernel.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -s` failed)"),
                ));
            }

            let kernel = core::str::from_utf8(&kernel.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            if !arch.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -k` failed)"),
                ));
            }
            let arch = core::str::from_utf8(&arch.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            let sys = if sys.status.success() {
                let sys = core::str::from_utf8(&sys.stdout)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .trim()
                    .to_string();
                Some(sys)
            } else {
                None
            };

            Ok(Uname { kernel, arch, sys })
        }),
        windows => ({
            let kernel = std::process::Command::new("uname").arg("-s").output();

            let kernel = match kernel{
                Ok(kernel) => kernel,
                Err(_) => {
                    return Ok(Uname{kernel: format!("Windows NT"), arch: std::env::var("PROCESSOR_ARCHITECTURE").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e)), sys: None})
                }
            }

            let arch = std::process::Command::new("uname").arg("-m").output()?;
            let sys = std::process::Command::new("uname").arg("-o").output()?;

            if !kernel.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -s` failed)"),
                ));
            }

            let kernel = core::str::from_utf8(&kernel.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            if !arch.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -k` failed)"),
                ));
            }
            let arch = core::str::from_utf8(&arch.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            let sys = if sys.status.success() {
                let sys = core::str::from_utf8(&sys.stdout)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .trim()
                    .to_string();
                Some(sys)
            } else {
                None
            };

            Ok(Uname { kernel, arch, sys })
        })
        _ => panic!("We don't know how to determine target system name. This is a bug if lccc has host support on this target")
    }
}
