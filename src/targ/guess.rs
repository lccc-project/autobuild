use target_tuples::Target;

use crate::log::{dbg, log, log_debug, trace, LogLevel};

pub mod uname;

macro_rules! maybe_stringify {
    ($tt:literal) => {
        $tt
    };
    ($tt:ident) => {
        ::core::stringify!($tt)
    };
}

macro_rules! opt_stringify {
    () => {
        ::core::option::Option::None
    };
    (*) => {
        _
    };
    ($tt:tt) => {
        ::core::option::Option::Some(maybe_stringify!($tt))
    };
}

macro_rules! gen_guess_table{
    {
        $($match_kernel:tt, $match_arch:tt $(,$match_sys:tt)? => $targ_arch:ident-$targ_vendor:ident-$targ_os:ident$(-$targ_env:ident)?;)*
    } => {
        fn guess_from_uname(uname: &uname::Uname) -> ::target_tuples::Target{
            match (&*uname.kernel, &*uname.arch, uname.sys.as_deref()){
                $((maybe_stringify!($match_kernel),maybe_stringify!($match_arch), opt_stringify!($($match_sys)?)) => {
                    let arch = target_tuples::Architecture::parse(::core::stringify!($targ_arch));
                    let vendor = target_tuples::Vendor::parse(::core::stringify!($targ_vendor));
                    let os = target_tuples::OS::parse(::core::stringify!($targ_os));
                    let env = opt_stringify!($($targ_env)?).map(target_tuples::Environment::parse);

                    ::target_tuples::Target::from_components(arch,vendor, Some(os), env, None)
                })*
                (kernel,arch,sys) => panic!("Could not identify host target. If the target is supported as a host target by lccc, this is a bug. If you know the name of the target, you can file an issue report. The name string is kernel={}, arch={}, os={}",
                            kernel,arch,sys.unwrap_or("<not provided>"))
            }
        }
    }
}

with_builtin_macros::with_builtin! {
    let $data = include_from_root!("src/targ/guess.data") in {
        gen_guess_table!($data);
    }
}

pub fn ident_target() -> Target {
    trace!(ident_target);
    let uname = uname::uname().expect("Could not determine the system names for the host target. If the target is supported as a host target by lccc, this is a bug.");

    dbg!(guess_from_uname(&uname))
}
