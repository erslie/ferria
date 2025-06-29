use std::{fs::File, io::BufReader, path::Path};
use ratatui::crossterm::event::read;
use rodio::{Decoder, Source};
use id3::{Tag, TagLike};
use std::time::Duration;
use std::io::{Error, ErrorKind};

use crate::error::FerriaError;

#[derive(Clone)]
pub struct AudioTrackMetaData {
    pub title: String,
    pub artist: String,
    pub duration: Option<Duration>,
}

pub struct AudioTrack {
    pub decoder: Decoder<BufReader<File>>,
    pub metadata: AudioTrackMetaData,
}

impl AudioTrack {

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, FerriaError> {

        //ここは取れなくてもok
        let mut metadata = read_id3_metadata(&path).unwrap();

        let reader = load_mp3(&path)?;

        let decoder = decode_audio_from_reader(reader)?;

        metadata.duration = decoder.total_duration();

        Ok(AudioTrack { decoder, metadata })

    }

}

pub fn load_mp3<P: AsRef<Path>>(path: P) -> Result<BufReader<File>, FerriaError> {

    let path_ref = path.as_ref();

    if !path_ref.exists() {
        return Err(FerriaError::IOError(Error::new(ErrorKind::NotFound,format!("File not found: {}", path_ref.display()))));
    }

    if !path_ref.is_file() {
        return Err(FerriaError::IOError(Error::new(ErrorKind::InvalidData, format!("Path is not a file: {}", path_ref.display()))));
    }

    let file: File = File::open(path_ref)
    .map_err(|_e| FerriaError::IOError(Error::new(ErrorKind::Other, format!("Failed to open file: {}", path_ref.display()))))?;

    Ok(BufReader::new(file))

}

fn decode_audio_from_reader(reader: BufReader<File>) -> Result<Decoder<BufReader<File>>, crate::error::FerriaError> {

    Decoder::new(reader)
    .map_err(|e| crate::error::FerriaError::AudioError(format!("Failed to decoder audio: {}", e)))

}

pub fn read_id3_metadata<P: AsRef<Path>>(path: P) -> Result<AudioTrackMetaData, FerriaError> {

    //id3タグの有無を問わないようにする(cdからの吸い出し以外では無いことが多々ある)
    let tag = Tag::read_from_path(&path).ok();

    Ok( AudioTrackMetaData { 
        title: tag.as_ref().and_then(|t| t.title().map(|s| s.to_string())).unwrap_or_else(|| "Unknown Title".to_string()), 
        artist: tag.as_ref().and_then(|t| t.artist().map(|s| s.to_string())).unwrap_or_else(|| "Unknown Artist".to_string()),   
        duration: None, } )

}

#[cfg(test)]
mod test_loader {

}


