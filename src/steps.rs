use std::{
    ffi::{CString, OsStr},
    io::Write,
};

use authkit::{AuthnFlags, BaseFlags, CredAction, Pam};

use crate::{
    config::{NIRI_GREETER_CONFIG, NIRI_SESSION_CONFIG},
    niri,
};

use super::{Error, Result};

macro_rules! try_cstrings {
    ($($s:expr),* $(,)?) => {
        (|| -> core::result::Result<_, std::ffi::NulError> {
            Ok(($(std::ffi::CString::new($s)?,)*))
        })()
    };
}

macro_rules! forke {
    ($ENV:expr, $($arg:expr),* $(,)?) => {{
        let bin = std::ffi::CString::new(std::env::current_exe()?.to_str().ok_or(crate::error::Error::ToStrError)?)?;

        let args = [
            &bin,
            $(&std::ffi::CString::new($arg)?,)*
        ];

        match unsafe { nix::unistd::fork() } {
            Ok(nix::unistd::ForkResult::Parent { child }) => child,
            Ok(nix::unistd::ForkResult::Child) => {
                if nix::unistd::execve::<&std::ffi::CString, std::ffi::CString>(&bin, &args, $ENV).is_err() {
                    std::process::exit(1);
                }

                unreachable!("SOMETHING BAD HAPPENED")
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

macro_rules! fork {
    ($($arg:expr),* $(,)?) => {{
        let bin = std::ffi::CString::new(std::env::current_exe()?.to_str().ok_or(crate::error::Error::ToStrError)?)?;

        let args = [
            &bin,
            $(&std::ffi::CString::new($arg)?,)*
        ];

        match unsafe { nix::unistd::fork() } {
            Ok(nix::unistd::ForkResult::Parent { child }) => child,
            Ok(nix::unistd::ForkResult::Child) => {
                if nix::unistd::execv::<&std::ffi::CString>(&bin, &args).is_err() {
                    std::process::exit(1);
                }

                unreachable!("SOMETHING BAD HAPPENED")
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

fn get_current_user() -> Result<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|e| {
            eprintln!("{e}");
            Error::UnknownCurrentUserHost
        })
}

pub fn start_display_tty(tty_number: u16) -> Result<()> {
    println!("Starting RILM display in TTY mode on tty{}", tty_number);
    println!("Running as root on tty{}", tty_number);

    let mut txn = Pam::start("rilm".into(), "greeter".into(), "".into())?;
    txn.authenticate(AuthnFlags::empty())?;
    txn.account_management(AuthnFlags::empty())?;

    txn.items_mut().set_tty_name(Some(&OsStr::new("tty3")))?;
    txn.env_mut().insert("XDG_VTNR", "3");

    txn.env_mut().insert("XDG_SEAT", "seat0");
    txn.env_mut().insert("XDG_SESSION_CLASS", "greeter");
    txn.env_mut().insert("USER", "greeter");
    txn.env_mut().insert("LOGNAME", "greeter");
    txn.env_mut().insert("HOME", "");
    txn.env_mut().insert("SHELL", "/bin/bash");
    txn.env_mut().insert("TERM", "linux");

    txn.open_session(BaseFlags::empty())?;
    txn.setcred(CredAction::Establish)?;

    // let fd = authkit::tty::open(3)?;
    // let current = authkit::tty::current(&fd);
    // if current != 3 {
    //     authkit::tty::switch(&fd, 3);
    // }

    // authkit::tty::take(&fd);

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

    let child = forke!(&env, "start", "greeter");

    loop {
        match nix::sys::wait::waitpid(child, None) {
            Err(nix::errno::Errno::EINTR) => continue,
            Err(e) => {
                eprintln!("waitpid failed: {e}");
                break;
            }
            Ok(_) => break,
        }
    }

    // authkit::tty::switch(&fd, current);
    // authkit::tty::close(fd);

    txn.close_session(BaseFlags::empty())?;

    todo!(
        r#"
            - [ ] Check for a user named 'greeter'. If not present create it and add it to the video group.
            - [ ] Open a UnixSocket only readable for root, writable for user "greeter"
            - [ ] Open a PAM session for user "greeter"
            - [ ] Fork, exec this program again with commands 'rilm start greeter --user "greeter"'
            - ...
            - [ ] Wait for it to finish
            - ...
            - [ ] End the PAM session
            - [ ] Once it's done we read the socket, there should be a pair of (user, cred),
            - [ ] Close the socket
            - [ ] Open a PAM session for this user, add credentials to it
            - [ ] Fork, exec this program again with commands 'rilm start session --user <user>'
            - ...
            - [ ] Wait for it to finish
            - ...
            - [ ] End the PAM session
            - [ ] Once it's done repeat this program by running 'rilm start tty' (or 'rilm start display tty')
            "#
    )
}

pub fn start_display_winit() -> Result<()> {
    let current_user = get_current_user()?;
    println!("Starting RILM display in Winit mode");
    println!("Running in simulated window under user: {}", current_user);

    let child = fork!("start", "greeter");

    loop {
        match nix::sys::wait::waitpid(child, None) {
            Err(nix::errno::Errno::EINTR) => continue,
            Err(e) => {
                eprintln!("waitpid failed: {e}");
                break;
            }
            Ok(_) => break,
        }
    }

    todo!(
        r#"
            - [ ] Open a UnixSocket only readable for <current_user>, writable for user <current_user>
            - [x] Fork, exec this program again with commands 'rilm start greeter'
            - ...
            - [ ] Wait for UnixSocket pair of login/cred
            - [ ] If ok, then send close signal wait for close
            - [ ] If not, then send retry signal, repeat
            - ...
            - [ ] Once it's done we read the socket, there should be a pair of (user, cred),
            - [ ] Close the socket
            - [ ] Open a PAM authentication for this user, add credentials to it
            - [ ] Fork, exec this program again with commands 'rilm start session'
            - ...
            - [ ] Wait for it to finish
            - ...
            - [ ] Once it's done repeat this program by running 'rilm start winit' (or 'rilm start display tty')
            "#
    )
}

pub fn start_greeter(user: Option<String>) -> Result<()> {
    let current_user = get_current_user()?;

    println!(
        "Starting RILM greeter for user: {}",
        user.as_ref().unwrap_or(&current_user)
    );

    if let Some(username) = user {
        let user = nix::unistd::User::from_name(&username)
            .ok()
            .flatten()
            .ok_or(Error::UnknownUserWithName(username.clone()))?;

        let cuser = std::ffi::CString::new(username.as_str()).map_err(Error::NulError)?;

        nix::unistd::initgroups(&cuser, user.gid).map_err(Error::UserError)?;
        nix::unistd::setgid(user.gid).map_err(Error::UserError)?;
        nix::unistd::setuid(user.uid).map_err(Error::UserError)?;
    }

    let bin = std::env::current_exe()?;

    niri::launch!(
        NIRI_GREETER_CONFIG,
        bin.to_str().ok_or(crate::error::Error::ToStrError)?,
        "start",
        "greeter",
        "--prompt"
    );

    todo!(
        r#"
            < parent process should have filled the correct ENV vars to open a niri session >

            - [x] if <user> != None, change for this user
            - [x] do an `execv` with niri and the greeter config
            "#
    )
}

pub fn start_greeter_prompt() -> Result<()> {
    print!("login: ");
    std::io::stdout().flush()?;
    let mut login = String::new();
    std::io::stdin().read_line(&mut login)?;
    let login = login.trim_end();

    println!("Your login is: {}", login);

    std::thread::park();

    todo!(
        r#"
            - [ ] ask for login / credentials
            - [ ] send to the root this pair on a unixsocket
            - [ ] park until the root closes the greeter
            "#
    )
}

pub fn start_session(user: Option<String>) -> Result<()> {
    let current_user = get_current_user()?;

    println!(
        "Starting RILM session for user: {}",
        user.as_ref().unwrap_or(&current_user)
    );

    if let Some(username) = user {
        let user = nix::unistd::User::from_name(&username)
            .ok()
            .flatten()
            .ok_or(Error::UnknownUserWithName(username.clone()))?;

        let cuser = std::ffi::CString::new(username.as_str()).map_err(Error::NulError)?;

        nix::unistd::initgroups(&cuser, user.gid).map_err(Error::UserError)?;
        nix::unistd::setgid(user.gid).map_err(Error::UserError)?;
        nix::unistd::setuid(user.uid).map_err(Error::UserError)?;
    }

    niri::launch!(NIRI_SESSION_CONFIG, "/usr/bin/nu");

    todo!(
        r#"
            < parent process should have filled the correct ENV vars to open a niri session >

            - [x] if <user> != None, change for this user
            - [x] do an `execv` with niri and the session config
            "#
    )
}
