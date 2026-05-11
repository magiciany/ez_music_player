use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "next")]
    Next,
    #[serde(rename = "prev")]
    Prev,
    #[serde(rename = "add")]
    Add { path: String },
    #[serde(rename = "remove")]
    Remove { index: usize },
    #[serde(rename = "move")]
    Move { from: usize, to: usize },
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "list")]
    List,
    #[serde(rename = "shuffle")]
    Shuffle,
    #[serde(rename = "repeat")]
    Repeat { mode: RepeatMode },
    #[serde(rename = "quit")]
    Quit,
    #[serde(rename = "status")]
    Status,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    Off,
    One,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum PlaybackState {
    Stopped,
    Playing { track_index: usize },
    Paused { track_index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub items: Vec<QueueItem>,
    pub current: Option<usize>,
    pub repeat: RepeatMode,
    pub shuffle: bool,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            current: None,
            repeat: RepeatMode::Off,
            shuffle: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}