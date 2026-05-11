use music_common::{Command, Response};
use crate::audio::AudioEngine;
use crate::state::PlayerState;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Handlers {
    pub state: Arc<Mutex<PlayerState>>,
    pub audio: Arc<AudioEngine>,
}

impl Handlers {
    pub async fn handle(&self, cmd: Command, id: u64) -> Response {
        match cmd {
            Command::Play => {
                let mut state = self.state.lock().await;
                if let Some(path) = state.current_track() {
                    self.audio.play_file(&path).await.ok();
                    state.playback = music_common::PlaybackState::Playing {
                        track_index: state.queue.current.unwrap_or(0),
                    };
                } else if let Some(idx) = state.queue.next_index() {
                    let path = state.queue.items[idx].path.clone();
                    self.audio.play_file(&path).await.ok();
                    state.queue.current = Some(idx);
                    state.playback = music_common::PlaybackState::Playing { track_index: idx };
                }
                Response { id, ok: true, error: None, body: None }
            }
            Command::Pause => {
                self.audio.pause().await;
                Response { id, ok: true, error: None, body: None }
            }
            Command::Stop => {
                self.audio.stop().await;
                Response { id, ok: true, error: None, body: None }
            }
            Command::Next => {
                let mut state = self.state.lock().await;
                if let Some(path) = state.play_next() {
                    self.audio.play_file(&path).await.ok();
                }
                Response { id, ok: true, error: None, body: None }
            }
            Command::Prev => {
                let mut state = self.state.lock().await;
                if let Some(path) = state.play_prev() {
                    self.audio.play_file(&path).await.ok();
                }
                Response { id, ok: true, error: None, body: None }
            }
            Command::Add { path } => {
                let mut state = self.state.lock().await;
                state.queue.add(path);
                Response { id, ok: true, error: None, body: None }
            }
            Command::Remove { index } => {
                let mut state = self.state.lock().await;
                state.queue.remove(index);
                Response { id, ok: true, error: None, body: None }
            }
            Command::Move { from, to } => {
                let mut state = self.state.lock().await;
                state.queue.move_item(from, to);
                Response { id, ok: true, error: None, body: None }
            }
            Command::Clear => {
                let mut state = self.state.lock().await;
                state.queue.clear();
                Response { id, ok: true, error: None, body: None }
            }
            Command::List => {
                let state = self.state.lock().await;
                let body = serde_json::to_value(&state.queue).ok();
                Response { id, ok: true, error: None, body }
            }
            Command::Shuffle => {
                let mut state = self.state.lock().await;
                state.queue.shuffle();
                Response { id, ok: true, error: None, body: None }
            }
            Command::Repeat { mode } => {
                let mut state = self.state.lock().await;
                state.set_repeat(mode);
                Response { id, ok: true, error: None, body: None }
            }
            Command::Quit => {
                std::process::exit(0);
            }
            Command::Status => {
                let state = self.state.lock().await;
                let body = serde_json::json!({
                    "playback": &state.playback,
                    "repeat": &state.queue.repeat,
                    "shuffle": state.queue.shuffle,
                    "current": state.queue.current,
                });
                Response { id, ok: true, error: None, body: Some(body) }
            }
        }
    }
}