use std::ffi::NulError;

use nix::errno::Errno;

#[derive(Debug)]
pub enum Error {
    UnknownCurrentUserHost,
    UnknownUserWithName(String),
    NulError(NulError),
    UserError(Errno),
    IoError(std::io::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCurrentUserHost => write!(f, "RILM was ran under unknown user."),
            Self::UnknownUserWithName(name) => {
                write!(f, "RILM was provided an unknown username {name}.")
            }
            Self::NulError(e) => write!(f, "{e}"),
            Self::UserError(e) => write!(f, "{e}"),
            Self::IoError(e) => write!(f, "{e}"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
