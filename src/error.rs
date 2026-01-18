use std::ffi::NulError;

use nix::errno::Errno;

#[derive(Debug)]
pub enum Error {
    UnknownCurrentUserHost,
    UnknownUserWithName(String),
    NulError(NulError),
    UserError(Errno),
    IoError(std::io::Error),
    AuthenticationError(authkit::ErrorCode),
    ToStrError,
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
            Self::AuthenticationError(e) => write!(f, "{e}"),
            Self::ToStrError => write!(f, "to_str() error"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<authkit::ErrorCode> for Error {
    fn from(value: authkit::ErrorCode) -> Self {
        Error::AuthenticationError(value)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(value: std::ffi::NulError) -> Self {
        Error::NulError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<Errno> for Error {
    fn from(value: Errno) -> Self {
        Self::UserError(value)
    }
}
