use clap::{Parser, Subcommand, ValueEnum};

mod config;
mod niri;
mod steps;

#[derive(Parser, Debug)]
#[command(
    name = "rilm",
    about = "RILM launcher",
    version,
    subcommand_required = false
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start rilm
    Start(StartArgs),
    /// Patch rilm configuration (may need sudo)
    PatchConfig,
}

#[derive(Parser, Debug)]
struct StartArgs {
    /// How to start rilm
    #[command(subcommand)]
    mode: StartMode,
}

#[derive(Subcommand, Debug)]
enum StartMode {
    /// Start rilm inside a tty session
    Tty {
        /// TTY number (must be >= 1)
        #[arg(
            default_value_t = 1,
            value_parser = validate_tty_number
        )]
        tty_number: u16,
    },
    /// Start rilm as a winit window
    Winit,
    /// Start rilm as logni
    Logni {
        /// Username for logni
        #[arg(long, default_value = "logni")]
        user: String,
    },
    /// Start rilm session
    Session {
        /// Username for session
        #[arg(long)]
        user: String,
    },
}

fn validate_tty_number(s: &str) -> Result<u16, String> {
    let num: u16 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if num < 1 {
        return Err(String::from("TTY number must be >= 1"));
    }
    Ok(num)
}

fn main() {
    use steps::*;

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Start(start_args)) => match start_args.mode {
            StartMode::Tty { tty_number } => {
                start_tty(tty_number);
            }
            StartMode::Winit => {
                start_winit();
            }
            StartMode::Logni { user } => {
                start_logni(&user);
            }
            StartMode::Session { user } => {
                start_session(&user);
            }
        },
        Some(Command::PatchConfig) => {
            patch_config();
        }
        None => {
            // Par défaut, démarrer en mode TTY 1
            start_tty(1);
        }
    }

    todo!(
        r#"Commands:
        - rilm patch-config (may need sudo)

        - rilm start tty
        - rilm start winit

        - rilm start logni --user <username> (default = 'logni')
        - rilm start session --user <username>
        "#
    )
}

// fn start_tty_root() {
//     println!("Starting rilm inside a TTY session");

//     todo!(
//         r#"
//         - In this process we're root
//         - Open a UnixSocket only readable for root, writable for user "logni"
//         - Open a PAM session for user "logni"
//         - Fork, exec this program again with commands 'rilm start tty logni'
//         - ...
//         - Wait for it to finish
//         - ...
//         - End the PAM session
//         - Once it's done we read the socket, there should be a pair of (user, cred),
//         - Close the socket
//         - Open a PAM session for this user, add credentials to it
//         - Fork, exec this program again with commands 'rilm start tty user <name_of_user>'
//         - ...
//         - Wait for it to finish
//         - ...
//         - End the PAM session
//         - Once it's done repeat this program by running 'rilm start tty' (or 'rilm start tty root')
//         "#
//     )
// }

// fn start_tty_logni() {
//     println!("Starting rilm logni TTY");
//     todo!(
//         r#"
//         - Change for user logni
//         - parent process should have filled the correct ENV vars to open a wayland session
//         - do `niri::launch_greeter`
//         "#
//     )
// }

// fn start_tty_user() {
//     println!("Starting rilm user TTY");
//     todo!(
//         r#"
//         - In this new process we're still root
//         - Change for user <user>
//         - do `niri::launch_session`
//         "#
//     )
// }

// fn start_winit() {
//     println!("Starting rilm as a winit window");

//     todo!(
//         r#"
//         - Open a UnixSocket only readable for root, writable for user "provided user"
//         - Open a PAM session for user "provided user"
//         - Fork, exec this program again with commands 'rilm start logni'
//         - ...
//         - Wait for it to finish
//         - ...
//         - End the PAM session
//         - Once it's done we read the socket, there should be a pair of (user, cred),
//         - Close the socket
//         - Open a PAM session for this user, add credentials to it
//         - Fork, exec this program again with commands 'rilm start tty user <name_of_user>'
//         - ...
//         - Wait for it to finish
//         - ...
//         - End the PAM session
//         - Once it's done repeat this program by running 'rilm start tty' (or 'rilm start tty root')
//         "#
//     )
// }

// fn patch_config() {
//     println!("Patching rilm config");
// }
