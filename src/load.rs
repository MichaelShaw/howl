use lewton::inside_ogg::OggStreamReader;

use std::fs;
use std::fs::File;
use std::path::{Path};
use std::io;

use ogg;
use errors::*;

use HowlResult;

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

pub fn file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let meta_data = fs::metadata(path)?;
    Ok(meta_data.len())
}

pub enum LoadedSound {
    Static(Sound),
    Streaming(OggStreamReader<File>),
}

pub fn load_combined(path: &Path, streaming_size: u64) -> HowlResult<LoadedSound> {
    let size = file_size(path).chain_err(||format!("Checking size for {:?}", path))?;
    if size > streaming_size {
        let stream = load_ogg_stream(path)?;
        Ok(LoadedSound::Streaming(stream))
    } else {
        let sound = load_ogg(path)?;
        Ok(LoadedSound::Static(sound))
    }
}

pub fn load_ogg_stream(path: &Path) -> HowlResult<OggStreamReader<File>> {
    let f = File::open(path).chain_err(|| format!("Attempting to open path {:?}", path))?;
    let packet_reader = ogg::PacketReader::new(f);
	let srr = OggStreamReader::new(packet_reader).chain_err(|| format!("Attempting to open packet reader for {:?}", path))?;
    Ok(srr)
}


pub fn load_ogg(path: &Path) -> HowlResult<Sound> {
    let f = File::open(path).chain_err(|| format!("Attempting to open path {:?}", path))?;

    let packet_reader = ogg::PacketReader::new(f);
	let mut srr = OggStreamReader::new(packet_reader).chain_err(|| format!("Attempting to open packet reader for {:?}", path))?;
    
    if srr.ident_hdr.audio_channels > 2 {
        let err: Error = ErrorKind::TooManyChannels.into();
        return Err(err).chain_err(|| format!("Attempting to open packet reader for {:?}", path));
	}

    let mut data : Vec<i16> = Vec::new();
    while let Some(pck_samples) = srr.read_dec_packet_itl().chain_err(||format!("Attempting to read ogg packet for {:?}", path))? {
        data.extend(pck_samples.iter());
    }
    
    Ok(Sound {
        data: data,
        sample_rate: srr.ident_hdr.audio_sample_rate,
        channels: srr.ident_hdr.audio_channels,
    })
}
