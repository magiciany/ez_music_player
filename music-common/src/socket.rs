use std::path::PathBuf;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct SocketClient {
    path: PathBuf,
}

impl SocketClient {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub async fn connect(&self, cmd: &super::Command) -> super::Response {
        let mut stream = UnixStream::connect(&self.path).await
            .expect("daemon not running — start with `musicd`");
        let json = serde_json::to_string(cmd).unwrap();
        stream.write_all(json.as_bytes()).await.unwrap();
        stream.write_all(b"\n").await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        serde_json::from_slice(&buf).unwrap()
    }
}