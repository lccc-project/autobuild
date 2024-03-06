use std::io;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Uname {
    pub kernel: String,
    pub arch: String,
    pub sys: Option<String>,
    pub kver: String,
    pub krelease: String,
    pub hostname: String,
}

#[allow(unused_parens)] // cfg_match is a macro that exists
pub fn uname() -> io::Result<Uname> {
    cfg_match::cfg_match! {
        unix => ({
            let kernel = std::process::Command::new("uname").arg("-s").output()?;
            let arch = std::process::Command::new("uname").arg("-m").output()?;
            let sys = std::process::Command::new("uname").arg("-o").output()?;
            let kver = std::process::Command::new("uname").arg("-v").output()?;
            let krelease = std::process::Command::new("uname").arg("-r").output()?;
            let hostname = std::process::Command::new("uname").arg("-n").output()?;

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

            if !kver.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -k` failed)"),
                ));
            }
            let kver = core::str::from_utf8(&kver.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            if !krelease.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -k` failed)"),
                ));
            }
            let krelease = core::str::from_utf8(&krelease.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            if !hostname.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Could not determine target system name (executing `uname -k` failed)"),
                ));
            }
            let hostname = core::str::from_utf8(&hostname.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .to_string();


            Ok(Uname { kernel, arch, sys, kver, krelease, hostname})
        }),
        windows => ({
            use windows_sys::Win32::System::{SystemInformation, WindowsProgramming};

            let mut hostname = Vec::with_capacity(WindowsProgramming::MAX_COMPUTERNAME_LENGTH);
            let mut n = hostname.capacity() as u32;

            loop{
                if SystemInformation::GetComputerNameExA(SystemInformation::ComputerNameDnsHostname, hostname.as_mut_ptr(), &mut n){
                    hostname.set_len(n as usize);
                    break
                }
                hostname.reserve(n as usize);
            }

            let hostname = String::from_utf8(hostname)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;


            let mut os_ver_info = unsafe{core::mem::zeroed::<SystemInformation::OSVERSIONINFEXOW>()};
            os_ver_info.dwOSVersionInfoSize = core::mem::size_of::<SystemInformation::OSVERSIONINFEXOW>() as u32;

            let sys = std::process::Command::new("uname").arg("-o").output();

            if unsafe{SystemInformation::GetOsVersionExW(&mut os_ver_info)} {
                return Err(io::Error::new(io::Error::InvalidData, "Failed to run `GetOsVersionExW`"));
            }

            let pos = os_ver_info.szCSDVewrsion.find(0);

            let sp = String::from_utf16_lossy(&os_ver_info.szCSDVewrsion[..pos]);

            let mut krelease = format!("NT/{}.{}.{}", os_ver_info.dwMajorVersion, os_ver_info.dwMinorVersion, os_ver_info.dwBuildNumber);


            let product_ty = match os_ver_info.wProductType{
                0x01 => Cow::Borrowed("Workstation"),
                0x02 => Cow::Borrowed("Domain Controller"),
                0x03 => Cow::Borrowed("Server"),
                ty => Cow::Owned(format!("**Unknown product type 0x{:02X}**", ty))
            };

            let win_ver = match (os_ver_info.dwMajorVersion, os_ver_info.dwMinorVersion, os_ver_info.wProductType){
                (5,0,_) => Cow::Borrowed("Windows 2000"),
                (5,1,1) => Cow::Borrowed("Windows XP"),
                (5,2,1) => Cow::Borrowed("Windows XP Professional"),
                (5,2,2|3) => Cow::Borrowed("Windows Server 2003"),
                (6,0,1) => Cow::Borrowed("Windows Vista"),
                (6,0|1,2|3) => Cow::Borrowed("Windows Server 2008"),
                (6,1,1) => Cow::Borrowed("Windows 7"),
                (6,2|3,2|3) => Cow::Borrowed("Windows Server 2012"),
                (6,2,1) => Cow::Borrowed("Windows 8"),
                (6,3,1) => Cow::Borrowed("Windows 8.1"),
                (10,0,1) => Cow::Borrowed("Windows 10"),
                (10,0,2|3) => Cow::Borrowed("Windows Server 2016"),
                (maj,min, _) => Cow::Owned(format!("Windows NT {}.{}", maj,min))
            };

            let kver = format!("Microsoft {} {} {}", win_ver, sp, product_ty);

            let sys = match sys{
                Ok(sys) if sys.status.success() => {
                    let sys = core::str::from_utf8(&sys.stdout)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .trim()
                    .to_string();
                    Some(sys)
                }
                _ => None
            };

            Ok(Uname{
                kernel: format!("Windows NT"),
                arch: std::env::var("PROCESSOR_ARCHITECTURE").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?,
                sys,
                krelease,
                kver,
                hostname,
            })
        }),
        _ => panic!("We don't know how to determine target system name. This is a bug if lccc has host support on this target")
    }
}
