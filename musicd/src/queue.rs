use music_common::{Queue, QueueItem, RepeatMode};
use std::path::PathBuf;
use std::fs;

fn queue_path() -> PathBuf {
    dirs::config_dir().unwrap().join("music/queue.json")
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