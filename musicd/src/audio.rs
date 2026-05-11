use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

const DSD_FORMATS: &[&str] = &["dsf", "dff"];

pub struct AudioEngine {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
}

fn is_dsd(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| DSD_FORMATS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

impl AudioEngine {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink: Arc::new(Mutex::new(sink)),
        }
    }

    pub async fn play_file(&self, path: &str) -> Result<(), String> {
        let path = Path::new(path);
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        if is_dsd(path) {
            return Err("DSD playback via ffmpeg hybrid not yet implemented".to_string());
        }

        let decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
        let sink = self.sink.lock().await;
        sink.stop();
        sink.append(decoder);
        sink.play();
        Ok(())
    }

    pub async fn pause(&self) {
        let sink = self.sink.lock().await;
        sink.pause();
    }

    pub async fn resume(&self) {
        let sink = self.sink.lock().await;
        sink.play();
    }

    pub async fn stop(&self) {
        let sink = self.sink.lock().await;
        sink.stop();
    }

    pub async fn is_playing(&self) -> bool {
        let sink = self.sink.lock().await;
        !sink.is_paused() && !sink.empty()
    }
}
