use music_common::{PlaybackState, RepeatMode, Queue};

pub struct PlayerState {
    pub playback: PlaybackState,
    pub queue: Queue,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playback: PlaybackState::Stopped,
            queue: Queue::load(),
        }
    }

    pub fn play_next(&mut self) -> Option<String> {
        if let Some(idx) = self.queue.next_index() {
            self.queue.current = Some(idx);
            self.playback = PlaybackState::Playing { track_index: idx };
            Some(self.queue.items[idx].path.clone())
        } else {
            self.playback = PlaybackState::Stopped;
            None
        }
    }

    pub fn play_prev(&mut self) -> Option<String> {
        if let Some(idx) = self.queue.prev_index() {
            self.queue.current = Some(idx);
            self.playback = PlaybackState::Playing { track_index: idx };
            Some(self.queue.items[idx].path.clone())
        } else {
            None
        }
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.queue.repeat = mode;
        self.queue.save();
    }

    pub fn current_track(&self) -> Option<String> {
        self.queue.current.map(|i| self.queue.items[i].path.clone())
    }
}