
use rodio::{OutputStream, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::path::Path;

use crate::error::FerriaError;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
    status: Arc<Mutex<PlaybackStatus>>,
    current_file_path: Arc<Mutex<Option<String>>>,
}

impl AudioPlayer {

    pub fn new() -> Result<Self, FerriaError> {

        let (_stream, stream_handle)  = OutputStream::try_default()
        .map_err(|e| FerriaError::AudioError(format!("Failed to get audio stream: {}", e)))?;

        let sink = Sink::try_new(&stream_handle)
        .map_err(|e| FerriaError::AudioError(format!("Failed to create audio sink: {}", e)))?;

        Ok(AudioPlayer {
            _stream,
            sink: Arc::new(Mutex::new(sink)),
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
            current_file_path: Arc::new(Mutex::new(None)), 
        })
        
    }

    pub fn play(&self, path_str: &str) -> Result<(), FerriaError> {

        let path = Path::new(path_str);

        if !path.exists() {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound,format!("File not found: {}", path_str)).into());
        }

        let file = File::open(path)?;

        Ok(())
        
    }
}

#[cfg(test)]
mod test {
    use crate::{audio::player::AudioPlayer, error::FerriaError};


    #[test]
    fn test_Audio_new() {
        assert!(AudioPlayer::new().is_ok());
    }

    #[test]
    fn test_Audio_play() {

        let audio = AudioPlayer::new().unwrap();

        let a = AudioPlayer::play(&audio, "a");

        assert!(a.is_ok());

    }

} 
