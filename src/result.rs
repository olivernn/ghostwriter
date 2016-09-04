extern crate git2;

use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum HookError {
    NoRemoteHost,
    Io(io::Error),
    Git(git2::Error),
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HookError::Io(ref e) => write!(f, "IO Error: {}", e),
            HookError::Git(ref e) => write!(f, "Git Error: {}", e),
            HookError::NoRemoteHost => write!(f, "No Remote Host"),
        }
    }
}

impl error::Error for HookError {
    fn description(&self) -> &str {
        match *self {
            HookError::Io(ref e) => e.description(),
            HookError::Git(ref e) => e.description(),
            HookError::NoRemoteHost => "no remote host",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HookError::Io(ref e) => Some(e),
            HookError::Git(ref e) => Some(e),
            HookError::NoRemoteHost => None,
        }
    }
}

impl From<io::Error> for HookError {
    fn from(e: io::Error) -> HookError {
        HookError::Io(e)
    }
}

impl From<git2::Error> for HookError {
    fn from(e: git2::Error) -> HookError {
        HookError::Git(e)
    }
}

pub enum Check {
    Pass,
    Fail(Vec<git2::Oid>),
}
