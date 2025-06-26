use std::{fs::File, io::BufReader, path::Path};
use rodio::Decoder;
use id3::{Tag, TagLike};
use std::time::Duration;
use std::io::{Error, ErrorKind};

use crate::error::FerriaError;

pub struct AudioTrackMetaData {
    pub title: String,
    pub artist: String,
    pub duration: Option<Duration>,
}

pub struct AudioTrack {
    pub decoder: Decoder<BufReader<File>>,
    pub metadata: AudioTrackMetaData,
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
    .map_err(|e| FerriaError::IOError(Error::new(ErrorKind::Other, format!("Failed to open file: {}", path_ref.display()))))?;

    Ok(BufReader::new(file))

}

pub fn read_id3_metadata<P: AsRef<Path>>(path: P, total_duration: Option<Duration>) -> Result<AudioTrackMetaData, FerriaError> {

    let tag = Tag::read_from_path(path).map_err(|e| FerriaError::IOError(Error::new(ErrorKind::Other, format!("Failed to read tag from audio: {}", e))))?;

    Ok( AudioTrackMetaData { 
        title: tag.title().unwrap_or("Unknown").into(), 
        artist: tag.artist().unwrap_or("Unknown").into(), 
        duration: total_duration, } )

}

#[cfg(test)]
mod test_loader {

    use std::{fs::File, io::BufReader, time::Duration};

    use crate::{audio::loader::{load_mp3, read_id3_metadata, AudioTrackMetaData}, error::FerriaError};


    #[test]
    fn test_load_mp3() {

        let path = std::path::Path::new("asset/test.mp3");

        let mp3 = load_mp3(path).or_else(|e| {eprintln!("{}", e); Err(e)} );

        assert!(mp3.is_ok())

    }

    #[test]
    fn test_read_id3_metadata() {

        //フリーmp3にid3タグがなかったけど実際はメタデータなくても再生ができればよしとする
        let path = std::path::Path::new("assets/eine.mp3");

        let meta = read_id3_metadata(path, None).or_else(|e| {eprintln!("{}", e); Err(e)});

        println!("{}", meta.as_ref().unwrap().title);
        println!("{}", meta.as_ref().unwrap().artist);

    }

}


