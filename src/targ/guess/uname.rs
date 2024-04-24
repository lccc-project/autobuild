use std::io;

use crate::log::{dbg, log, log_debug, trace, LogLevel};

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
    trace!(uname);
    cfg_match::cfg_match! {
        unix => ({
            use std::ffi::CStr;
            use core::mem::MaybeUninit;
            log_debug!(LogLevel::Trace, "{{uname}}: unix");

            let mut name = MaybeUninit::zeroed();

            if unsafe{libc::uname(name.as_mut_ptr())} < 0{
                return Err(dbg!(io::Error::last_os_error()));
            }

            let name = unsafe{name.assume_init()};


            let kernel = CStr::from_bytes_until_nul(bytemuck::cast_slice(&name.sysname)).expect("libc::uname should have inserted a NULL");
            let kernel = kernel.to_str().map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, format!("expected UTF-8 only in \"{:?}\"", kernel))))?.to_string();
            let hostname = CStr::from_bytes_until_nul(bytemuck::cast_slice(&name.nodename)).expect("libc::uname should have inserted a NULL");
            let hostname = hostname.to_str().map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, format!("expected UTF-8 only in \"{:?}\"", hostname))))?.to_string();
            let kver = CStr::from_bytes_until_nul(bytemuck::cast_slice(&name.version)).expect("libc::uname should have inserted a NULL");
            let kver = kver.to_str().map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, format!("expected UTF-8 only in \"{:?}\"", kver))))?.to_string();
            let krelease = CStr::from_bytes_until_nul(bytemuck::cast_slice(&name.release)).expect("libc::uname should have inserted a NULL");
            let krelease = krelease.to_str().map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, format!("expected UTF-8 only in \"{:?}\"", krelease))))?.to_string();

            let arch = CStr::from_bytes_until_nul(bytemuck::cast_slice(&name.machine)).expect("libc::uname should have inserted a NULL");
            let arch = arch.to_str().map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, format!("expected UTF-8 only in \"{:?}\"", arch))))?.to_string();

            log!(LogLevel::Info, "uname -o");
            let sys = if let Ok(cmd) = std::process::Command::new("uname").arg("-o").output(){
                if cmd.status.success(){
                    let st = String::from_utf8(cmd.stdout)
                        .map_err(|_| dbg!(io::Error::new(io::ErrorKind::InvalidData, "expected UTF-8 only in os name")))?;

                    Some(st.trim().to_string())
                }else{
                    None
                }
            }else{
                None
            };

            dbg!(Ok(Uname { kernel, arch, sys, kver, krelease, hostname}))
        }),
        windows => ({
            use windows_sys::Win32::System::{SystemInformation, WindowsProgramming};
            log_debug!(LogLevel::Trace, "{{uname}}: windows");

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

            dbg!(Ok(Uname{
                kernel: format!("Windows NT"),
                arch: std::env::var("PROCESSOR_ARCHITECTURE").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?,
                sys,
                krelease,
                kver,
                hostname,
            }))
        }),
        target_os = "lilium" => ({
            use lilium_sys::info;
            use lilium_sys::result::Error as LiliumError;
            log_debug!(LogLevel::Trace, "{{uname}}: lilium");
            let info = match info::RequestBuilder::new().request::<info::ArchInfo>().request::<info::OsVersion>().request::<info::KernelVendor>().opt_request::<info::ComputerName>().resolve(){
                Ok(responses) => responses,
                Err(LiliumError::InvalidOption | LiliumError::UnsupportedKernelFunction) => Err(io::Error::new(io::ErrorKind::Unsupported, "could not determine system name"))?,
                Err(LiliumError::InsufficientLength | LiliumError::InvalidMemory) => panic!("Bad info `FromRequest` impl"),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unknown error from `GetSystemInfo`: {:?}", e)))?
            };

            let arch = info.get::<info::ArchInfo>();
            let os_ver = info.get::<info::OsVersion>();
            let kvendor = info.get::<info::KernelVendor>();
            let cname = info.opt_get::<info::ComputerName>();

            let arch = match arch.arch_id{
                info::arch_info::ARCH_TYPE_X86_64 => format!("x86_64"),
                info::arch_info::ARCH_TYPE_X86_IA_32 => format!("i{}86", arch.version),
                info::arch_info::ARCH_TYPE_CLEVER_ISA => format!("clever"),
                info::arch_info::ARCH_TYPE_ARM32 => format!("arm"),
                info::arch_info::ARCH_TYPE_AARCH64 => format!("aarch64"),
                info::arch_info::ARCH_TYPE_RISCV32 => format!("riscv32"),
                info::arch_info::ARCH_TYPE_RISCV64 => format!("riscv64"),
                id => format!("**unknown arch {}**", id)
            };

            let kernel = format!("Lilium");
            let sys = os_ver.vendor;

            let krelease = format!("{} {}.{}", kvendor.vendor, kvendor.major_version, kvendor.minor_version);

            let kver = format!("{} {}.{}/{} (build id {})", os_ver.vendor, os_ver.major_version, os_ver.minor_version, kvendor.vendor, kvendor.build_id);

            let hostname = match cname{
                Some(cname) => cname.hostname,
                None => "".to_string()
            };

            dbg!(Ok(Uname{
                kernel,
                sys,
                krelease,
                kver,
                hostname,
            }))
        }),
        _ => panic!("We don't know how to determine target system name. This is a bug if lccc has host support on this target")
    }
}
