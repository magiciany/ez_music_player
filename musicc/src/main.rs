use clap::{Parser, Subcommand, CommandFactory};
use music_common::{Command, RepeatMode};
use music_common::socket::SocketClient;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    dirs::config_dir().unwrap().join("ez_music_player/musicd.sock")
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
        Commands::Add { path } => {
            let absolute = std::fs::canonicalize(&path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(path);
            Command::Add { path: absolute }
        }
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
            print!("{}", Cli::command().render_help());
            std::process::exit(0);
        }
    };
    let client = SocketClient::new(socket_path());
    let resp = client.connect(&cmd).await;
    if resp.ok {
        if let Some(body) = resp.body {
            if body.get("items").is_some() {
                let items = body.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let current = body.get("current").and_then(|v| v.as_u64()).map(|v| v as usize);
                let repeat = body.get("repeat").and_then(|v| v.as_str()).unwrap_or("off");
                let shuffle = body.get("shuffle").and_then(|v| v.as_bool()).unwrap_or(false);

                for (i, item) in items.iter().enumerate() {
                    let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let marker = if Some(i) == current { " [playing]" } else { "" };
                    println!("{}: {}{}", i, path, marker);
                }
                if !items.is_empty() {
                    println!();
                }
                println!("Repeat: {}  |  Shuffle: {}", repeat, shuffle);
            } else {
                println!("{}", serde_json::to_string_pretty(&body).unwrap());
            }
        }
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
}