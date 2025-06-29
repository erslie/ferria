use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

use ferria::{audio::{analyzer::{AudioAnalyzer, SpectrumData}, loader::AudioTrack, player::{AudioPlayer, PlaybackStatus}}, error::FerriaError};

fn main() -> Result<(), FerriaError> {

    println!("Ferria を起動します...");

    let player = AudioPlayer::new()?;
    println!("AudioPlayer が初期化されました。");

    let audio_file_path = PathBuf::from("assets/eine.mp3");

    println!("オーディオトラックをロード中: {:?}", audio_file_path);

    let audio_track = AudioTrack::new(&audio_file_path)?;

    println!("オーディオトラックのロードが完了しました。タイトル:{:?}, アーティスト:{:?}", audio_track.metadata.title, audio_track.metadata.artist);

    let (sample_tx, sample_rx) = mpsc::channel::<f32>();
    let (spectrum_tx, spectrum_rx) = mpsc::channel::<SpectrumData>();

    player.play(audio_track, Some(sample_tx))?;

    //sample_rateは実際はAudioPlayerのOutputStreamから取得したものを使用
    let _ = AudioAnalyzer::run_in_thread(1024, 44100, sample_rx, spectrum_tx);

    println!("再生を開始しました。");

    let total_duration = player.get_current_metadata().and_then(|m| m.duration);

    if let Some(duration) = total_duration {
        println!("再生時間: {}秒", duration.as_secs());

        let start_time = std::time::Instant::now();


        loop {
            let status = player.get_status();
            let elapsed = start_time.elapsed();
            println!("現在のステータス: {:?}, 経過時間: {:?}", status, elapsed);

            if status == PlaybackStatus::Stopped && elapsed > Duration::from_secs(1) {
                println!("再生が終了しました");
                break;
            }
            if elapsed > duration + Duration::from_secs(2) {
                println!("再生が予想時間を超えました。強制終了します。");
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
    else {
        println!("曲の総再生時間が不明です。50秒間再生後、終了します。");
        thread::sleep(Duration::from_secs(50));
        player.stop();
        println!("再生を停止します。");
    }

    println!("Ferria 終了します。");

    Ok(())

}

