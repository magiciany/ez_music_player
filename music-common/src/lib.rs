use std::path::PathBuf;
use std::fs;

use serde::{Deserialize, Serialize};

pub mod socket;

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

fn queue_path() -> PathBuf {
    dirs::config_dir().unwrap().join("ez_music_player/queue.json")
}

impl Queue {
    pub fn load() -> Self {
        let path = queue_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = queue_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(&path, content).ok();
    }

    pub fn add(&mut self, path: String) {
        self.items.push(QueueItem { path });
        self.save();
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.items.remove(index);
            if let Some(cur) = self.current {
                if cur == index {
                    self.current = None;
                } else if cur > index {
                    self.current = Some(cur - 1);
                }
            }
            self.save();
            true
        } else {
            false
        }
    }

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from < self.items.len() && to < self.items.len() && from != to {
            let item = self.items.remove(from);
            self.items.insert(to, item);
            // Update current index
            self.current = match self.current {
                None => None,
                Some(cur) => {
                    if cur == from {
                        Some(to)
                    } else if from < cur && to >= cur {
                        Some(cur - 1)
                    } else if from > cur && to <= cur {
                        Some(cur + 1)
                    } else {
                        Some(cur)
                    }
                }
            };
            self.save();
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current = None;
        self.save();
    }

    pub fn shuffle(&mut self) {
        let current_path = self.current.map(|i| self.items[i].path.clone());
        for i in (1..self.items.len()).rev() {
            let j = rand::random::<usize>() % (i + 1);
            self.items.swap(i, j);
        }
        if let Some(ref cp) = current_path {
            self.current = self.items.iter().position(|item| &item.path == cp);
        }
        self.save();
    }

    pub fn next_index(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        match self.current {
            None => Some(0),
            Some(i) => {
                let next = i + 1;
                if next < self.items.len() {
                    Some(next)
                } else if self.repeat == RepeatMode::All {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    pub fn prev_index(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        match self.current {
            None => Some(self.items.len() - 1),
            Some(i) => {
                if i > 0 {
                    Some(i - 1)
                } else if self.repeat == RepeatMode::All {
                    Some(self.items.len() - 1)
                } else {
                    Some(i)
                }
            }
        }
    }
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