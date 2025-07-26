use std::{fmt::format, io::Error, path::PathBuf, sync::mpsc, thread, time::Duration};
use ferria::{app::FerriaApp, visualizer, audio::{analyzer::{AudioAnalyzer, SpectrumData}, loader::AudioTrack, player::{AudioPlayer, PlaybackStatus}} ,error::FerriaError};


fn main() -> Result<(), FerriaError> {

    let mut app = FerriaApp::new()?;
    app.run()?;

    Ok(())
}
