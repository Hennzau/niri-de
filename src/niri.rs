macro_rules! launch {
    ($config:expr, $($arg:expr),* $(,)?) => {{
        use std::{io::Write, os::unix::ffi::OsStrExt};

        let mut tmp = tempfile::NamedTempFile::new().map_err(Error::IoError)?;
        tmp.write_all($config.as_bytes()).map_err(Error::IoError)?;

        let (niri, c, config, dash, alacritty, e) = try_cstrings!(
            "/usr/bin/niri",
            "-c",
            tmp.path().as_os_str().as_bytes(),
            "--",
            "/usr/bin/alacritty",
            "-e",
        )
        .map_err(Error::NulError)?;

        let args = [
            &niri, &c, &config, &dash, &alacritty, &e,
            $(&std::ffi::CString::new($arg)?,)*
        ];

        if nix::unistd::execv::<&std::ffi::CString>(&niri, &args).is_err() {
            std::process::exit(1);
        }
    }};
}
pub(crate) use launch;
