use authkit::{
    AuthnFlags, BaseFlags, ConversationAdapter, CredAction, Demux, Pam, PamBuilder,
    Result as PamResult,
};

use std::ffi::{CString, OsStr, OsString};

struct UsernamePassConvo {
    username: String,
    password: String,
}

impl ConversationAdapter for UsernamePassConvo {
    fn prompt(&self, _: impl AsRef<OsStr>) -> PamResult<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _: impl AsRef<OsStr>) -> PamResult<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn error_msg(&self, _: impl AsRef<OsStr>) {}

    fn info_msg(&self, _: impl AsRef<OsStr>) {}
}

fn authenticate(username: &str, password: &str) -> PamResult<Pam<Demux<UsernamePassConvo>>> {
    let user_pass = UsernamePassConvo {
        username: username.into(),
        password: password.into(),
    };

    let mut txn = PamBuilder::new("greetd-greeter")
        .username(username)
        .build(user_pass.into_conversation())?;

    txn.authenticate(AuthnFlags::empty())?;
    txn.account_management(AuthnFlags::empty())?;

    Ok(txn)
}

fn main() {
    println!("====================");
    println!("CURRENT ENV:");
    for (key, val) in std::env::vars() {
        println!("\t{:?}={:?}", key, val);
    }
    println!("====================");

    if let Ok(mut txn) = authenticate("root", "") {
        println!("Logged IN {:?}", txn.username(None));

        txn.items_mut()
            .set_tty_name(Some(&OsStr::new("tty3")))
            .expect("Coudln't set PAM to tty3");

        txn.environ_mut().insert("XDG_VTNR", "3");
        txn.environ_mut().insert("XDG_SEAT", "seat0");
        txn.environ_mut().insert("XDG_SESSION_CLASS", "greeter");
        txn.environ_mut().insert("USER", "root");
        txn.environ_mut().insert("LOGNAME", "root");
        txn.environ_mut().insert("HOME", "/home/root");
        txn.environ_mut().insert("SHELL", "/bin/bash");
        txn.environ_mut().insert("TERM", "linux");

        txn.open_session(BaseFlags::empty())
            .expect("Couldn't open a session");

        txn.setcred(CredAction::Establish).expect("Can't set cred");

        println!("PAM ENV:");
        let env = txn
            .environ()
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

        for var in &env {
            println!("\t{var:?}");
        }

        let terminal = CString::new("/dev/tty3").unwrap();
        let fd = authkit::tty::open(&terminal).expect("Couldn't open terminal");
        if authkit::tty::current(&fd) != 3 {
            authkit::tty::switch(&fd, 3);
        }

        authkit::tty::take(&fd);

        let niri = CString::new("/usr/local/bin/niri").unwrap();
        let c = CString::new("-c").unwrap();
        let arg = CString::new("/usr/local/share/niri-de/niri-lm.kdl").unwrap();
        nix::unistd::execve(&niri, &[&niri, &c, &arg], &env).ok();

        txn.close_session(BaseFlags::empty())
            .expect("Couldn't stop session");
    }
}
