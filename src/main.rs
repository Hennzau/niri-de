use clap::{Parser, Subcommand};
use error::*;

mod error;

mod config;
mod niri;
mod steps;

#[derive(Parser, Debug)]
#[command(
    name = "rilm",
    about = "RILM: Niri Desktop Environment",
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
    /// What to start
    #[command(subcommand)]
    target: StartTarget,
}

#[derive(Subcommand, Debug)]
enum StartTarget {
    /// Start display manager
    Display {
        /// Display mode
        #[command(subcommand)]
        mode: DisplayMode,
    },
    /// Start greeter (will use current user if --user not specified)
    Greeter {
        /// Username for greeter (optional, defaults to current user)
        #[arg(long)]
        user: Option<String>,

        /// If set, will launch a prompt asking for credentials
        #[arg(long)]
        prompt: bool,
    },
    /// Start session (will use current user if --user not specified)
    Session {
        /// Username for session (optional, defaults to current user)
        #[arg(long)]
        user: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum DisplayMode {
    /// Start display in TTY mode (assumes root, runs everything on tty number, default = 1)
    Tty {
        /// TTY number (must be >= 1)
        #[arg(
            default_value_t = 1,
            value_parser = validate_tty_number
        )]
        tty_number: u16,
    },
    /// Start display in Winit mode (run everything in a simulated window under the current user)
    Winit,
}

fn validate_tty_number(s: &str) -> core::result::Result<u16, String> {
    let num: u16 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if num < 1 {
        return Err(String::from("TTY number must be >= 1"));
    }
    Ok(num)
}

fn cli() -> Result<()> {
    use steps::*;

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Start(start_args)) => match start_args.target {
            StartTarget::Display { mode } => match mode {
                DisplayMode::Tty { tty_number } => start_display_tty(tty_number),
                DisplayMode::Winit => start_display_winit(),
            },
            StartTarget::Greeter { user, prompt } => {
                if prompt {
                    start_greeter_prompt()
                } else {
                    start_greeter(user)
                }
            }
            StartTarget::Session { user } => start_session(user),
        },
        Some(Command::PatchConfig) => patch_config(),
        None => start_display_tty(1),
    }
}

fn main() {
    if let Err(e) = cli() {
        eprintln!("{e}");
    }
}

fn patch_config() -> Result<()> {
    println!("Patching RILM configuration (may require sudo)");

    todo!(
        r#"
            - Patch pam.d service
            - Patch systemctl service
        "#
    )
}
