use std::{io::Write, os::unix::ffi::OsStrExt};
use tempfile::NamedTempFile;

use crate::config::NIRI_LM_CONFIG;

pub const NIRI: &'static str = "/usr/bin/niri";

macro_rules! try_cstrings {
    ($($s:expr),* $(,)?) => {
        (|| -> Result<_, std::ffi::NulError> {
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

pub fn launch_greeter() {
    let mut tmp = NamedTempFile::new().expect("Couldn't create a temp file");
    tmp.write_all(NIRI_LM_CONFIG.as_bytes())
        .expect("Couldn't write config in temp file");

    let (niri, c, config) = try_cstrings!(NIRI, "-c", tmp.path().as_os_str().as_bytes())
        .expect("Couldn't parse command");

    exec!(&niri, &[&niri, &c, &config])
}
