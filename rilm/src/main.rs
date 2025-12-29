use clap::{Parser, Subcommand, ValueEnum};

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

    /// Patch rilm configuration
    PatchConfig,
}

#[derive(Parser, Debug)]
struct StartArgs {
    /// How to start rilm
    #[arg(
        value_enum,
        default_value_t = StartMode::Tty
    )]
    mode: StartMode,
}

#[derive(ValueEnum, Clone, Debug)]
enum StartMode {
    /// Start rilm inside a tty session
    Tty,

    /// Start rilm as a winit window
    Winit,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // `rilm` → start tty
        None => start_tty(),

        Some(Command::Start(args)) => match args.mode {
            StartMode::Tty => start_tty(),
            StartMode::Winit => start_winit(),
        },

        Some(Command::PatchConfig) => patch_config(),
    }
}

fn start_tty() {
    println!("Starting rilm inside a TTY session");
}

fn start_winit() {
    println!("Starting rilm as a winit window");
}

fn patch_config() {
    println!("Patching rilm config");
}
