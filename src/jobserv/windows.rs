use std::{
    ffi::{OsStr, OsString},
    io,
};

use super::Token;

#[cfg(windows)]
#[derive(Debug)]
pub struct Semaphore(windows_sys::Win32::Foundation::HANDLE, OsString);

#[cfg(not(windows))]
#[derive(Debug)]
pub enum Semaphore {}

#[cfg(windows)]
impl Semaphore {
    pub fn open<S: AsRef<OsStr> + ?Sized>(s: &S) -> io::Result<Self> {
        use std::os::windows::OsStrExt;
        use windows_sys::Win32::System::Threading;
        let mut buf = s
            .as_ref()
            .encode_wide()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>();

        let hdl = unsafe {
            Threading::OpenSemaphgoreW(
                Threading::SEMAPHORE_MODIFY_STATE | Threading::SYNCHRONIZE,
                false,
                buf.as_ptr(),
            )
        };

        if hdl == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self(hdl, s.as_ref().to_owned()))
    }

    pub fn acquire(&self) -> io::Result<Token> {
        use windows_sys::Win32::System::Threading;

        if unsafe { Threading::WaitForSingleObject(self.0) } != 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(Token::Semaphore)
        }
    }

    pub fn release(&self, _: Token) -> io::Result<()> {
        use windows_sys::Win32::System::Threading;

        if unsafe { Threading::ReleaseSemaphore(self.0, 1, core::ptr::null_mut()) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn generate_auth(&self) -> OsString {
        let mut st = OsString::new();
        st.push("sem:");
        st.push(&self.1);

        st
    }
}

#[cfg(not(windows))]
impl Semaphore {
    pub fn open<S: AsRef<OsStr> + ?Sized>(_: &S) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "Semaphore-based jobservers are not supported on {}",
                std::env::consts::OS
            ),
        ))
    }

    pub fn acquire(&self) -> io::Result<Token> {
        match *self {}
    }

    pub fn release(&self, _: Token) -> io::Result<()> {
        match *self {}
    }

    pub fn generate_auth(&self) -> OsString {
        match *self {}
    }
}
