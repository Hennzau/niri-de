use authkit::{AuthnFlags, BaseFlags, CredAction, Pam, Result as PamResult};

use std::{
    ffi::{CString, OsStr},
    io::Write,
};

use macros::*;
mod macros;

pub fn greeter() {
    login!("niri-lm");
    logger!("greeter");

    let niri = CString::new("/usr/bin/niri").unwrap();
    let c = CString::new("-c").unwrap();
    let arg = CString::new("/usr/local/share/niri-de/niri-lm.kdl").unwrap();
    exec!(&niri, &[&niri, &c, &arg])
}

pub fn main() {
    if let Some(arg) = std::env::args().nth(1) {
        if arg.as_str() == "--greeter" {
            greeter();
            return;
        }
    }

    logger!("main");

    fn authenticate(service: &str, username: &str, password: &str) -> PamResult<Pam> {
        let mut txn = Pam::start(service.into(), username.into(), password.into())?;

        txn.authenticate(AuthnFlags::empty())?;
        txn.account_management(AuthnFlags::empty())?;

        Ok(txn)
    }

    if let Ok(mut txn) = authenticate("niri-lm", "niri-lm", "") {
        log!("Logged IN {:?}", txn.username(None),);

        txn.items_mut()
            .set_tty_name(Some(&OsStr::new("tty4")))
            .expect("Coudln't set PAM to tty4");

        txn.env_mut().insert("XDG_VTNR", "4");
        txn.env_mut().insert("XDG_SEAT", "seat0");
        txn.env_mut().insert("XDG_SESSION_CLASS", "greeter");
        txn.env_mut().insert("USER", "niri-lm");
        txn.env_mut().insert("LOGNAME", "niri-lm");
        txn.env_mut().insert("HOME", "");
        txn.env_mut().insert("SHELL", "/bin/bash");
        txn.env_mut().insert("TERM", "linux");

        txn.open_session(BaseFlags::empty())
            .expect("Couldn't open a session");

        txn.setcred(CredAction::Establish).expect("Can't set cred");

        log!("Opening terminal",);
        let fd = authkit::tty::open(4).expect("Couldn't open terminal");
        let current = authkit::tty::current(&fd);
        if current != 4 {
            log!("Switching VT",);
            authkit::tty::switch(&fd, 4);
        }

        log!("Taking terminal",);
        authkit::tty::take(&fd);

        let env = txn
            .env()
            .iter()
            .map(|(key, val)| {
                CString::new(format!(
                    "{}={}",
                    key.to_str().unwrap(),
                    val.to_str().unwrap()
                ))
                .unwrap()
            })
            .collect::<Vec<_>>();

        let bin = std::env::current_exe().expect("Couldn't get current exe");
        let greeter = CString::new(bin.to_str().expect("Invalid path")).unwrap();
        let arg = CString::new("--greeter").unwrap();

        log!("Spawning greeter",);
        let child = fork!(&greeter, &[&greeter, &arg], &env);
        wait!(child);

        authkit::tty::switch(&fd, current);
        authkit::tty::close(fd);
    } else {
        std::process::exit(1)
    }
}
