mod audio;
mod handlers;
mod state;

use std::path::PathBuf;
use std::fs;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;

use music_common::Command;
use handlers::Handlers;
use state::PlayerState;
use audio::AudioEngine;

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("ez_music_player")
}

fn socket_path() -> PathBuf {
    config_dir().join("musicd.sock")
}

fn pid_path() -> PathBuf {
    config_dir().join("musicd.pid")
}

fn ensure_config_dir() {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).ok();
}

#[tokio::main]
async fn main() {
    ensure_config_dir();
    let sock_path = socket_path();

    // Check if already running
    if UnixStream::connect(&sock_path).await.is_ok() {
        eprintln!("musicd already running");
        std::process::exit(1);
    }
    // Remove stale socket file if exists
    if sock_path.exists() {
        std::fs::remove_file(&sock_path).ok();
    }

    // Write PID
    fs::write(&pid_path(), std::process::id().to_string()).ok();

    // Listen on socket
    let listener = UnixListener::bind(&sock_path).unwrap();

    let state = Arc::new(Mutex::new(PlayerState::new()));
    let audio = Arc::new(AudioEngine::new());
    let handlers = Handlers { state, audio };

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let mut stream = BufReader::new(stream);
            let mut line = String::new();
            while stream.read_line(&mut line).await.unwrap_or(0) > 0 {
                let json_str = line.trim();
                if json_str.is_empty() {
                    continue;
                }
                if let Ok(cmd) = serde_json::from_str::<Command>(json_str) {
                    let id = 1;
                    let resp = handlers.handle(cmd, id).await;
                    let resp_json = serde_json::to_string(&resp).unwrap();
                    stream.write_all(resp_json.as_bytes()).await.ok();
                    stream.write_all(b"\n").await.ok();
                }
                line.clear();
            }
        }
    }
}