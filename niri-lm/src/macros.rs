macro_rules! user {
    ($lit:literal) => {
        nix::unistd::User::from_name($lit)
            .expect("Error getting user by name")
            .expect("No user with this name")
    };
}

macro_rules! login {
    ($str:expr) => {
        let user = user!($str);
        let cuser = std::ffi::CString::new($str).expect("Couldn't make a CString");

        nix::unistd::initgroups(&cuser, user.gid).expect("Couldn't init groups");
        nix::unistd::setgid(user.gid).expect("Couldn't set GID");
        nix::unistd::setuid(user.uid).expect("Couldn't set UID");
    };
}

macro_rules! log {
    ($str:expr, $($arg:expr,)*) => {
        println!($str, $($arg,)*);
        std::io::stdout().flush().ok();
    };
}

macro_rules! logger {
    ($lit:literal) => {{
        let uid = nix::unistd::Uid::current();
        let gid = nix::unistd::Gid::current();
        let user = nix::unistd::User::from_uid(uid)
            .ok()
            .map(|user| user.map(|user| user.name))
            .flatten();

        log!("{} - User: {:?} with {} - {}", $lit, user, uid, gid,);
        for (key, value) in std::env::vars() {
            println!("\t{}={}", key, value);
        }
    }};
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

macro_rules! fork {
    ($PROG:expr, $ARGS:expr, $ENV:expr) => {
        match unsafe { nix::unistd::fork() } {
            Ok(nix::unistd::ForkResult::Parent { child }) => child,
            Ok(nix::unistd::ForkResult::Child) => {
                exec!($PROG, $ARGS, $ENV);
                unreachable!("SOMETHING BAD HAPPENED")
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };
    ($PROG:expr, $ARGS:expr) => {
        match unsafe { nix::unistd::fork() } {
            Ok(nix::unistd::ForkResult::Parent { child }) => child,
            Ok(nix::unistd::ForkResult::Child) => {
                exec!($PROG, $ARGS);
                unreachable!("SOMETHING BAD HAPPENED")
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };
}

macro_rules! wait {
    ($pid:expr) => {
        loop {
            match nix::sys::wait::waitpid($pid, None) {
                Err(nix::errno::Errno::EINTR) => continue,
                Err(e) => {
                    eprintln!("waitpid failed: {e}");
                    break;
                }
                Ok(_) => break,
            }
        }
    };
}

pub(crate) use exec;
pub(crate) use fork;
pub(crate) use log;
pub(crate) use logger;
pub(crate) use login;
pub(crate) use user;
pub(crate) use wait;
