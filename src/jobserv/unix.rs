use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::raw::c_int;

use crate::jobserv::Token;

pub struct PipePair {
    read: File,
    write: File,
    read_fd_no: c_int,
    write_fd_no: c_int,
}

impl PipePair {
    #[cfg(unix)]
    pub fn new(mut token_count: usize) -> io::Result<Self> {
        use std::os::fd::FromRawFd;

        let mut fds = [0; 2];

        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let [read_fd_no, write_fd_no] = fds;

        let mut buf = vec![b'P'; token_count];

        if unsafe { libc::write(write_fd_no, buf.as_mut_ptr().cast(), token_count as _) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let (read, write) = unsafe {
            (
                File::from_raw_fd(read_fd_no),
                File::from_raw_fd(write_fd_no),
            )
        };

        Ok(Self {
            read,
            write,
            read_fd_no,
            write_fd_no,
        })
    }

    #[cfg(not(unix))]
    pub fn new(token_count: usize) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "Bare pipe jobserver is not supported on {}",
                std::env::consts::OS
            ),
        ))
    }

    #[cfg(unix)]
    pub fn from_fds(read_fd_no: c_int, write_fd_no: c_int) -> io::Result<Self> {
        use std::os::fd::FromRawFd;

        if unsafe { libc::fcntl(read_fd_no, libc::F_GETFL) } == -1 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::fcntl(write_fd_no, libc::F_GETFL) } == -1 {
            return Err(io::Error::last_os_error());
        }

        let (read, write) = unsafe {
            (
                File::from_raw_fd(read_fd_no),
                File::from_raw_fd(write_fd_no),
            )
        };

        Ok(Self {
            read,
            write,
            read_fd_no,
            write_fd_no,
        })
    }

    #[cfg(not(unix))]
    pub fn from_fds(_: c_int, _: c_int) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "Bare pipe jobserver is not supported on {}",
                std::env::consts::OS
            ),
        ))
    }

    pub fn acquire(&self) -> io::Result<Token> {
        use std::io::Read;
        let mut read = &self.read;

        let mut b = 0;

        read.read_exact(core::slice::from_mut(&mut b))?;

        Ok(Token::Char(b))
    }

    pub fn release(&self, tok: Token) -> io::Result<()> {
        use std::io::Write;
        match tok {
            Token::Char(c) => {
                let mut write = &self.write;

                write.write_all(core::slice::from_ref(&c))
            }
            Token::Semaphore => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Semaphore tokens are not valid for ",
            )),
        }
    }

    pub fn generate_auth(&self) -> OsString {
        OsString::from(format!("{},{}", self.read_fd_no, self.write_fd_no))
    }
}
