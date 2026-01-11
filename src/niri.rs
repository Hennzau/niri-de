use std::{io::Write, os::unix::ffi::OsStrExt};
use tempfile::NamedTempFile;

use crate::error::{Error, Result};

macro_rules! try_cstrings {
    ($($s:expr),* $(,)?) => {
        (|| -> core::result::Result<_, std::ffi::NulError> {
            Ok(($(std::ffi::CString::new($s)?,)*))
        })()
    };
}

macro_rules! exec {
    ($PROG:expr, $ARGS:expr, $ENV:expr) => {
        if nix::unistd::execve::<&std::ffi::CStr, std::ffi::CString>($PROG, $ARGS, $ENV).is_err() {
            std::process::exit(1);
        }
    };

    ($PROG:expr, $ARGS:expr) => {
        if nix::unistd::execv::<&std::ffi::CStr>($PROG, $ARGS).is_err() {
            std::process::exit(1);
        }
    };
}

pub fn launch(config: &str) -> Result<()> {
    let mut tmp = NamedTempFile::new().map_err(Error::IoError)?;
    tmp.write_all(config.as_bytes()).map_err(Error::IoError)?;

    let (niri, c, config) = try_cstrings!("/usr/bin/niri", "-c", tmp.path().as_os_str().as_bytes())
        .map_err(Error::NulError)?;

    exec!(&niri, &[&niri, &c, &config]);

    Ok(())
}
