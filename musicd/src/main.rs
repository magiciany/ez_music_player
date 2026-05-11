mod audio;
mod handlers;
mod queue;
mod state;

use std::path::PathBuf;
use std::fs;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

use music_common::Command;
use handlers::Handlers;
use state::PlayerState;
use audio::AudioEngine;

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("music")
}

fn socket_path() -> PathBuf {
    config_dir().join("socket")
}

fn pid_path() -> PathBuf {
    config_dir().join("daemon.pid")
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
    if sock_path.exists() {
        if UnixStream::connect(&sock_path).await.is_ok() {
            eprintln!("musicd already running");
            std::process::exit(1);
        }
    }

    // Write PID
    fs::write(&pid_path(), std::process::id().to_string()).ok();

    // Listen on socket
    let listener = UnixListener::bind(&sock_path).unwrap();

    let state = Arc::new(Mutex::new(PlayerState::new()));
    let audio = Arc::new(AudioEngine::new());
    let handlers = Handlers { state, audio };

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await.ok();
            let json_str = String::from_utf8(buf).unwrap();
            if let Ok(cmd) = serde_json::from_str::<Command>(&json_str) {
                let id = 1; // TODO: extract id from message
                let resp = handlers.handle(cmd, id).await;
                let resp_json = serde_json::to_string(&resp).unwrap();
                stream.write_all(resp_json.as_bytes()).await.ok();
            }
        }
    }
}