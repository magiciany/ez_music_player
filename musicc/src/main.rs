use clap::{Parser, Subcommand};
use music_common::{Command, RepeatMode};
use music_common::socket::SocketClient;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    dirs::config_dir().unwrap().join("music/socket")
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Play,
    Pause,
    Stop,
    Next,
    Prev,
    Add { path: String },
    Remove { index: usize },
    Move { from: usize, to: usize },
    Clear,
    List,
    Shuffle,
    Repeat {
        #[arg(value_enum)]
        mode: RepeatModeArg,
    },
    Quit,
    Status,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum RepeatModeArg {
    Off,
    One,
    All,
}

impl From<RepeatModeArg> for RepeatMode {
    fn from(m: RepeatModeArg) -> Self {
        match m {
            RepeatModeArg::Off => RepeatMode::Off,
            RepeatModeArg::One => RepeatMode::One,
            RepeatModeArg::All => RepeatMode::All,
        }
    }
}

fn to_command(cmd: Commands) -> Command {
    match cmd {
        Commands::Play => Command::Play,
        Commands::Pause => Command::Pause,
        Commands::Stop => Command::Stop,
        Commands::Next => Command::Next,
        Commands::Prev => Command::Prev,
        Commands::Add { path } => Command::Add { path },
        Commands::Remove { index } => Command::Remove { index },
        Commands::Move { from, to } => Command::Move { from, to },
        Commands::Clear => Command::Clear,
        Commands::List => Command::List,
        Commands::Shuffle => Command::Shuffle,
        Commands::Repeat { mode } => Command::Repeat { mode: mode.into() },
        Commands::Quit => Command::Quit,
        Commands::Status => Command::Status,
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cmd = match cli.cmd {
        Some(c) => to_command(c),
        None => {
            eprintln!("Usage: musicc <command> [args]");
            std::process::exit(1);
        }
    };
    let client = SocketClient::new(socket_path());
    let resp = client.connect(&cmd).await;
    if resp.ok {
        if let Some(body) = resp.body {
            println!("{}", serde_json::to_string_pretty(&body).unwrap());
        }
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
}