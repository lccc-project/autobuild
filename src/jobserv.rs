#![allow(dead_code)]

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::path::Path;

use core::marker::PhantomData;

mod unix;
mod windows;

struct Fifo(File, OsString);

impl Fifo {
    pub fn open<S: AsRef<OsStr> + ?Sized>(fname: &S) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(false)
            .open(Path::new(fname))?;
        Ok(Self(file, fname.as_ref().to_owned()))
    }

    pub fn acquire(&self) -> io::Result<Token> {
        use std::io::Read;
        let mut read = &self.0;

        let mut b = 0;

        read.read_exact(core::slice::from_mut(&mut b))?;

        Ok(Token::Char(b))
    }

    pub fn release(&self, tok: Token) -> io::Result<()> {
        use std::io::Write;
        match tok {
            Token::Char(c) => {
                let mut write = &self.0;

                write.write_all(core::slice::from_ref(&c))
            }
            Token::Semaphore => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Semaphore tokens are not valid for fifo",
            )),
        }
    }

    pub fn generate_auth(&self) -> OsString {
        let mut st = OsString::new();
        st.push("fifo:");
        st.push(&self.1);

        st
    }
}

enum Connection {
    Fifo(Fifo),
    PipePair(unix::PipePair),
    Semaphore(windows::Semaphore),
}

enum Token {
    Char(u8),
    Semaphore,
}

impl Connection {
    pub fn parse_auth(x: &str) -> io::Result<Connection> {
        if let Some(val) = x.strip_prefix("fifo:") {
            Fifo::open(val).map(Self::Fifo)
        } else if let Some(val) = x.strip_prefix("sem:") {
            windows::Semaphore::open(val).map(Self::Semaphore)
        } else if let Some((l, _)) = x.split_once(':') {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unrecognized jobserver authentication type `{}`", l),
            ))
        } else if let Some((l, r)) = x.split_once(',') {
            let read = l.parse().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Expected an integer, got `{}`", l),
                )
            })?;
            let write = r.parse().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Expected an integer, got `{}`", r),
                )
            })?;
            unix::PipePair::from_fds(read, write).map(Self::PipePair)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid jobserver authentication format {}", x),
            ))
        }
    }

    pub fn generate_auth(&self) -> OsString {
        match self {
            Connection::Fifo(con) => con.generate_auth(),
            Connection::PipePair(con) => con.generate_auth(),
            Connection::Semaphore(con) => con.generate_auth(),
        }
    }

    pub fn acquire(&self) -> io::Result<Token> {
        match self {
            Connection::Fifo(con) => con.acquire(),
            Connection::PipePair(con) => con.acquire(),
            Connection::Semaphore(con) => con.acquire(),
        }
    }

    pub fn release(&self, tok: Token) -> io::Result<()> {
        match self {
            Connection::Fifo(con) => con.release(tok),
            Connection::PipePair(con) => con.release(tok),
            Connection::Semaphore(con) => con.release(tok),
        }
    }
}

pub struct JobServerConstructor<'a>(PhantomData<&'a mut &'a mut ()>);

impl JobServerConstructor<'static> {
    pub fn scope<F: for<'a> FnOnce(&'a mut JobServerConstructor<'a>)>(f: F) {
        let mut ctor = JobServerConstructor(PhantomData);

        f(&mut ctor)
    }
}

impl<'a> JobServerConstructor<'a> {}

pub struct JobServer<'a> {
    con: Connection,

    phantom_life_token: PhantomData<&'a mut &'a mut ()>,
}

impl<'a> JobServer<'a> {
    pub fn generate_auth(&self) -> OsString {
        self.con.generate_auth()
    }
}
