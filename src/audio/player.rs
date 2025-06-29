

use rodio::{Decoder, OutputStream, Sample, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
use std::path::{Path, PathBuf};

use crate::audio;
use crate::error::FerriaError;
use crate::audio::loader::{AudioTrack, AudioTrackMetaData};
use crate::audio::analyzer::SpectrumData;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

//rodio::Sourceのサンプルを別のチャネルに転送するためのラッパー
pub struct SampleForwarder<T>
where T: Source + Send + 'static,
      T::Item: rodio::Sample,
{
    inner: T,
    sender: mpsc::Sender<f32>,
}

impl<T> SampleForwarder<T>
where T: Source + Send + 'static,
      T::Item: rodio::Sample, {

    pub fn new(inner: T, sender: mpsc::Sender<f32>) -> Self {
        SampleForwarder { inner, sender }
    }
    
}

impl<T> Iterator for SampleForwarder<T>
where T: Source + Send + 'static,
      T::Item: rodio::Sample, 
{

    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {

        let sample = self.inner.next();

        if let Some(s) = sample {
            let _ = self.sender.send(s.to_f32());
        }

        sample
    }

}

impl<T> Source for SampleForwarder<T>
where T: Source + Send + 'static,
      T::Item: rodio::Sample,
{

    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }

    //next()はIteratorの実装で提供されるから不要

}



pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    status: Arc<Mutex<PlaybackStatus>>,
    current_file_path: Arc<Mutex<Option<PathBuf>>>,
    current_meta_data: Arc<Mutex<Option<AudioTrackMetaData>>>,
}

impl AudioPlayer {

    pub fn new() -> Result<Self, FerriaError> {

        let (_stream, stream_handle)  = OutputStream::try_default()
        .map_err(|e| FerriaError::AudioError(format!("Failed to get audio stream: {}", e)))?;

        let sink = Sink::try_new(&stream_handle)
        .map_err(|e| FerriaError::AudioError(format!("Failed to create audio sink: {}", e)))?;

        Ok(AudioPlayer {
            _stream,
            sink: sink,
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
            current_file_path: Arc::new(Mutex::new(None)), 
            current_meta_data: Arc::new(Mutex::new(None)),
        })
        
    }

    pub fn play(&self, audio_track: AudioTrack, analyzer_sender: Option<mpsc::Sender<f32>>) -> Result<(), FerriaError> {

        //再生中あったら停止
        self.sink.stop();
        //キューもクリア
        self.sink.clear();

        
        //TODO: スキップ、シークは今後実装予定

        let decoder = audio_track.decoder;

        if let Some(sender) = analyzer_sender {

            let forwarder = SampleForwarder::new(decoder, sender);

            self.sink.append(forwarder);

        }
        else {

            self.sink.append(decoder);

        }

        self.sink.play();

        *self.status.lock().unwrap() = PlaybackStatus::Playing;

        *self.current_file_path.lock().unwrap() = Some(PathBuf::from(audio_track.metadata.title.clone()));

        *self.current_meta_data.lock().unwrap() = Some(audio_track.metadata);

        Ok(())

    }

    pub fn pause(&self) {

        if *self.status.lock().unwrap() == PlaybackStatus::Playing {

            self.sink.pause();

            *self.status.lock().unwrap() = PlaybackStatus::Paused;

        }

    }

    pub fn resume(&self) {

        if *self.status.lock().unwrap() == PlaybackStatus::Paused {

            self.sink.play();

            *self.status.lock().unwrap() = PlaybackStatus::Playing;
        }
    }

    pub fn stop(&self) {

        self.sink.stop();

        self.sink.clear();

        *self.status.lock().unwrap() = PlaybackStatus::Stopped;

        *self.current_file_path.lock().unwrap() = None;

        *self.current_meta_data.lock().unwrap() = None;

    }

    pub fn get_status(&self) -> PlaybackStatus {

        let guard = self.status.lock().unwrap();

        (*guard).clone()

    }

    pub fn get_current_file_path(&self) -> Option<PathBuf> {
        self.current_file_path.lock().unwrap().clone()
    }

    pub fn get_current_metadata(&self) -> Option<AudioTrackMetaData> {
        self.current_meta_data.lock().unwrap().clone()
    }

}

#[cfg(test)]
mod test {

    use crate::audio::{loader, player};

    use super::*;
    use std::io::Cursor;
    use rodio::source::SineWave;
    use std::thread;
    
    fn load_mp3() -> AudioTrack {

        let path = PathBuf::from("assets/eine.mp3");

        if !path.exists() {
            panic!("Test MP3 file not found: {:?}" ,path);
        }

        crate::audio::loader::AudioTrack::new(&path).unwrap()

    }

    #[test]
    fn test_audio_player_new() {

        let player_result = AudioPlayer::new();
        assert!(player_result.is_ok());

        let player = player_result.unwrap();
        assert_eq!(player.get_status(), PlaybackStatus::Stopped);
        assert!(player.get_current_file_path().is_none());

    }

    #[test]
    fn test_play_pause_resume_stop() {

        let player = AudioPlayer::new().unwrap();
        let audio_track = load_mp3();

        assert_eq!(player.get_status(), PlaybackStatus::Stopped);

        player.play(audio_track, None).unwrap();
        assert_eq!(player.get_status(), PlaybackStatus::Playing);
        assert!(player.get_current_file_path().is_some());
        assert!(player.get_current_metadata().is_some());

        player.pause();
        assert_eq!(player.get_status(), PlaybackStatus::Paused);

        player.resume();
        assert_eq!(player.get_status(), PlaybackStatus::Playing);

        player.stop();
        assert_eq!(player.get_status(), PlaybackStatus::Stopped);
        assert!(player.get_current_file_path().is_none());
        assert!(player.get_current_metadata().is_none());

    }

} 
