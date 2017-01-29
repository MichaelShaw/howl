use lewton::inside_ogg::OggStreamReader;

use std::fs;
use std::fs::File;
use std::path::{Path};
use std::io;

use ogg;

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

// forked load based on file size?? 

pub fn file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let meta_data = try!(fs::metadata(path));
    Ok(meta_data.len())
}

pub enum LoadedSound {
    Static(Sound),
    Streaming(OggStreamReader<File>),
}

pub fn load_combined<P: AsRef<Path>>(path: P, streaming_size: u64) -> HowlResult<LoadedSound> {
    let size = try!(file_size(&path));
    if size > streaming_size {
        let stream = try!(load_ogg_stream(path));
        Ok(LoadedSound::Streaming(stream))
    } else {
        let sound = try!(load_ogg(path));
        Ok(LoadedSound::Static(sound))
    }
}

pub fn load_ogg_stream<P: AsRef<Path>>(path: P) -> HowlResult<OggStreamReader<File>> {
    let f = try!(File::open(path));
    let packet_reader = ogg::PacketReader::new(f);
	let srr = try!(OggStreamReader::new(packet_reader));
    Ok(srr)
}

pub fn load_ogg<P: AsRef<Path>>(path: P) -> HowlResult<Sound> {
    let f = try!(File::open(path));

	// Prepare the reading
    let packet_reader = ogg::PacketReader::new(f);
	let mut srr = try!(OggStreamReader::new(packet_reader));
    
    if srr.ident_hdr.audio_channels > 2 {
		// the openal crate can't process these many channels directly
        // std::vec::Vec<i16>
		println!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
	}

    // let mut len_play = 0.0;
    let mut data : Vec<i16> = Vec::new();
    while let Some(pck_samples) = try!(srr.read_dec_packet_itl()) {
        // println!("I got some shit {:?}", pck_samples);
        // len_play += pck_samples.len() as f32 / srr.ident_hdr.audio_sample_rate as f32;
        data.extend(pck_samples.iter());
    }
    
    Ok(Sound {
        data: data,
        sample_rate: srr.ident_hdr.audio_sample_rate,
        channels: srr.ident_hdr.audio_channels,
    })
}
