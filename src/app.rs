
use crate::audio::analyzer;
use crate::error::FerriaError;

use crate::audio::{
    player::{AudioPlayer, PlaybackStatus},
    loader::AudioTrack,
    analyzer::{SpectrumData,AudioAnalyzer},
};

use ratatui::crossterm::{event::{
    self, 
    Event, 
    KeyEventKind, 
    KeyEvent},
     terminal::{
        disable_raw_mode,
        enable_raw_mode,
    }};

use std::time::Duration;
use std::thread;
use std::path::PathBuf;
use std::sync::mpsc;

pub struct FerriaApp {
    player: AudioPlayer,
}

impl FerriaApp {

    pub fn new() -> Result<Self, FerriaError>{
        let player = AudioPlayer::new()?;
        Ok( FerriaApp{ player } )
    }


    pub fn run(&mut self) -> Result<(), FerriaError> {

        println!("Ferria を起動します...");


        let _terminal_restore_guard = TerminalRestoreGuard;

        let audio_file_path = PathBuf::from("assets/eine.mp3");
        println!("オーディオトラックをロード中: {:?}", audio_file_path);

        let audio_track = AudioTrack::new(&audio_file_path)?;
        println!("オーディオトラックのロードが完了しました。タイトル:{:?}, アーティスト:{:?}", audio_track.metadata.title, audio_track.metadata.artist);

        let (sample_tx, sample_rx) = mpsc::channel::<f32>();
        let (spectrum_tx, spectrum_rx) = mpsc::channel::<SpectrumData>();

        self.player.play(audio_track, Some(sample_tx))?;

        //sample_rateは実際はAudioPlayerのOutputStreamから取得したものを使用
        let handle_analyzer = AudioAnalyzer::run_in_thread(1024, 44100, sample_rx, spectrum_tx);

        println!("再生を開始しました。");

        let start_time = std::time::Instant::now();
        let total_duration = self.player.get_current_metadata().and_then(|m| m.duration);

        enable_raw_mode()?;

        loop {

            let status = self.player.get_status();
            let elapsed = start_time.elapsed();

            if let Some(duration) = total_duration {
                if elapsed > duration + Duration::from_secs(2) {
                    println!("再生が予想時間を超えました。強制終了します。");
                    break;
                }
            }

            // println!("現在のステータス: {:?}, 経過時間: {:?}", status, elapsed);

            if status == PlaybackStatus::Stopped {
                println!("再生が終了しました");
                break;
            }

            if event::poll(Duration::from_millis(50))? {

                let event = event::read()?;
                match event {
                    Event::Key(event) => if !self.handle_key_event(&event) {
                        break;
                    },
                    _ => {}, //リサイズなどは今後実装？
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        drop(_terminal_restore_guard);

        println!("Ferria 終了します。");

        Ok(())
    }

    //true->loop continue / false->break;
    fn handle_key_event(&self, event: &KeyEvent) -> bool {

        if event.kind != KeyEventKind::Press { return true };

        match event.code {
            event::KeyCode::Enter |
            event::KeyCode::Char(' ') |
            event::KeyCode::Char('p') => {
                push_key_turn(&self.player)
            },
            event::KeyCode::Char('s') => { 
                push_key_stop(&self.player)
            },
            event::KeyCode::Char('+') => {
                push_key_volume_up(&self.player)
            }
            event::KeyCode::Char('-') => {
                push_key_volume_down(&self.player)
            }
            event::KeyCode::Char('q') |
            event::KeyCode::Char('c') if event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                push_key_kill(&self.player)
            },
            _ => true,
        }

    }

}

pub fn push_key_turn(player: &AudioPlayer) -> bool {

    let current_status = player.get_status();
    match current_status {
        PlaybackStatus::Playing => {
            player.pause();
            println!("pause.");
        },
        PlaybackStatus::Paused => {
            player.resume();
            println!("restart.");
        },
        _ => {},
    }

    true
}

pub fn push_key_stop(player: &AudioPlayer) -> bool {
    player.stop();
    true
}
 
pub fn push_key_volume_up(player: &AudioPlayer) -> bool {
    player.volume_up();
    true
}

pub fn push_key_volume_down(player: &AudioPlayer) -> bool {
    player.volume_down();
    true
}

pub fn push_key_kill(player: &AudioPlayer) -> bool {
    player.stop();
    false
}

struct TerminalRestoreGuard;
impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {

        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
    }
}