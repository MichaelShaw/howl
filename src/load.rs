use lewton::inside_ogg::OggStreamReader;

use std::fs;
use std::fs::File;
use std::path::{Path};

use ogg;

use super::LoadResult;
use super::errors::*;


#[derive(Clone, Debug)]
pub struct Sound {
    pub data : Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

impl Sound {
    pub fn duration(&self) -> f32 {
        (self.data.len() as f32) / (self.sample_rate as f32)
    }
}

pub enum LoadedSound {
    Static(Sound),
    Streaming(OggStreamReader<File>),
}

fn file_size(path: &Path) -> LoadResult<u64> {
    let meta_data = fs::metadata(path).map_err(|ioe| LoadError { path: path.to_path_buf(), reason: LoadErrorReason::FileReadError(ioe) })?;
    Ok(meta_data.len())
}

fn open_file(path:&Path) -> LoadResult<File> {
   File::open(path).map_err(|ioe| LoadError { path: path.to_path_buf(), reason: LoadErrorReason::FileReadError(ioe) }) 
}

fn open_stream_reader(path: &Path, packet_reader: ogg::PacketReader<File>) -> LoadResult<OggStreamReader<File>> {
    OggStreamReader::new(packet_reader).map_err(|oe| LoadError { path: path.to_path_buf(), reason: LoadErrorReason::ReadOggError(oe) })
}

pub fn load_combined(path: &Path, streaming_size: u64) -> LoadResult<LoadedSound> {
    let size = file_size(path)?;
    if size > streaming_size {
        let stream = load_ogg_stream(path)?;
        Ok(LoadedSound::Streaming(stream))
    } else {
        let sound = load_ogg(path)?;
        Ok(LoadedSound::Static(sound))
    }
}

pub fn load_ogg_stream(path: &Path) -> LoadResult<OggStreamReader<File>> {
    let f = open_file(path)?; 
    let packet_reader = ogg::PacketReader::new(f);
	let srr = open_stream_reader(path, packet_reader)?;
    Ok(srr)
}

pub fn load_ogg(path: &Path) -> LoadResult<Sound> {
    let f = open_file(path)?;

    let packet_reader = ogg::PacketReader::new(f);
	let mut srr = open_stream_reader(path, packet_reader)?;
    
    if srr.ident_hdr.audio_channels > 2 {
        return Err(LoadError{ path: path.to_path_buf(), reason: LoadErrorReason::TooManyChannels });
	}

    let mut data : Vec<i16> = Vec::new();
    while let Some(pck_samples) = srr.read_dec_packet_itl().map_err(|oe| LoadError {path: path.to_path_buf(), reason: LoadErrorReason::ReadOggError(oe) })? {
        data.extend(pck_samples.iter());
    }
    
    Ok(Sound {
        data: data,
        sample_rate: srr.ident_hdr.audio_sample_rate,
        channels: srr.ident_hdr.audio_channels,
    })
}
