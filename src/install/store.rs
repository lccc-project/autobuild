use super::InstallDirs;

crate::serialize::helpers::impl_serde! {
    InstallDirs = default{
        prefix,
        exec_prefix as "exec-prefix",
        bindir,
        sbindir,
        libdir,
        libexecdir,
        includedir,
        datarootdir,
        datadir,
        mandir,
        docdir,
        infodir,
        localedir,
        localstatedir,
        runstatedir,
        sharedstatedir,
        sysconfdir,
    }
}
