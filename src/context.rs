use alto;
use alto::{Context, Buffer, SourceTrait};
use alto::{Mono, Stereo};

use std::sync::Arc;
use std::path::{PathBuf};

use super::load::{load_combined, LoadedSound, load_ogg, Sound};
use super::source::{Sources, SoundSource, StreamingSoundSource, SoundSourceLoan};

use {Gain, DistanceModel, SoundName, SoundEvent};
use {SoundProviderResult, PreloadResult, SoundEventResult};
use read_directory_paths;
use super::errors::*;
use super::Listener;

use aphid::HashMap;

use rand;
use rand::Rng;

pub struct SoundContext<'d> {
    pub context: &'d Context<'d>,
    pub rng: rand::XorShiftRng,
    pub path: String,
    pub extension: String,
    pub sources: Sources<'d>,
    pub buffers: HashMap<SoundName, Vec<SoundBuffer<'d>>>,
    pub stream_above_file_size: u64,
    pub stream_buffer_duration: f32,
    pub master_gain : Gain,
    pub distance_model : DistanceModel,
    pub listener : Listener,
}

pub struct SoundBuffer<'d> {
    pub inner : Arc<Buffer<'d, 'd>>,
    pub gain: Gain,
    pub duration: f32, // we could track last used .... could be interesting if nothing else
}

pub fn create_sound_context<'d>(context: &'d Context<'d>, path:&str, extension: &str, rng: rand::XorShiftRng, stream_above_file_size: u64, stream_buffer_duration: f32) -> SoundContext<'d> {
    // we should probably create our sources here
    SoundContext {
        context: context,
        rng: rng,
        path: String::from(path),
        extension: String::from(extension),
        sources: Sources {
            next_event: 0,
            sources: Vec::new(),
            streaming: Vec::new(),
        },
        buffers: HashMap::default(),
        stream_above_file_size: stream_above_file_size,
        stream_buffer_duration: stream_buffer_duration,
        master_gain: 1.0,
        distance_model: alto::DistanceModel::None,
        listener: Listener::default() ,
    }
}

impl<'d> SoundContext<'d> {
    pub fn set_gain(&mut self, gain: Gain) -> SoundProviderResult<()> {
        self.context.set_gain(gain)?;
        self.master_gain = gain;

        Ok(())
    }

    pub fn create(&mut self, static_count: usize, streaming_count: usize) -> SoundProviderResult<()> {
        for _ in 0..static_count {
            let source = self.context.new_static_source()?;
            self.sources.sources.push(SoundSource { inner: source, current_binding: None});
        }
        for _ in 0..streaming_count {
            let source = self.context.new_streaming_source()?;
            self.sources.streaming.push(StreamingSoundSource { inner: source, stream_reader: None, current_binding: None });
        }
        Ok(())
    }

    pub fn set_listener(&mut self, listener: Listener) -> SoundProviderResult<()> {
        self.context.set_position(listener.position)?;
        self.context.set_velocity(listener.velocity)?;
        self.context.set_orientation::<[f32; 3]>((listener.orientation_forward.into(), listener.orientation_up.into()))?;

        self.listener = listener;
        
        Ok(())
    }

    pub fn purge(&mut self) -> SoundProviderResult<()> {
        self.sources.purge()?;
        self.buffers.clear();
        Ok(())
    }

    pub fn set_distace_model(&mut self, distance_model: DistanceModel) -> SoundProviderResult<()> {
        self.context.set_distance_model(distance_model)?;
        self.distance_model = distance_model;
        Ok(())
    }

    // just convenience
    pub fn stop(&mut self, loan:SoundSourceLoan) -> SoundProviderResult<()> {
        if let Some(ref mut source) = self.sources.for_loan(loan) {
            source.stop()?;
        }
        Ok(())
    }

    pub fn full_sound_paths(&self, sound_name:&str) -> PreloadResult<Vec<PathBuf>> {
        // 1. look for a directory with that name
        let ogg_path = PathBuf::from(format!("{}/{}.{}", &self.path, sound_name, &self.extension));
        let directory_path = PathBuf::from(format!("{}/{}", &self.path, sound_name));

        if ogg_path.exists() {
            Ok(vec![ogg_path])
        } else if directory_path.is_dir() {
            let mut sound_paths = Vec::new();
            let paths = read_directory_paths(&directory_path)?;
            for path in paths {
                if let Some(extension) = path.extension().and_then(|p| p.to_str()).map(|s|s.to_lowercase()) {
                    if extension == self.extension {
                        sound_paths.push(path);
                    }
                }
            }
//            println!("ok loading multi paths {:?}", sound_paths);
            Ok(sound_paths)
        } else {
            Ok(vec![])
        }
    }

    pub fn preload(&self, sound_name: &str, gain:Gain) -> PreloadResult<()> {
        let paths = self.full_sound_paths(sound_name);
//        context.buffers.insert(sound_name, buffers);
        Ok(())
    }

    pub fn buffer_sound(&self, sound: Sound, gain:Gain) -> PreloadResult<SoundBuffer<'d>> {
        let mut buffer = try!(self.context.new_buffer());
        let duration = sound.duration();
        if sound.channels == 1 {
            try!(buffer.set_data::<Mono<i16>, _>(sound.data, sound.sample_rate as i32));
        } else if sound.channels == 2 {
            try!(buffer.set_data::<Stereo<i16>, _>(sound.data, sound.sample_rate as i32));
        } else {
            // bail!(ErrorKind::TooManyChannels);
        }

        Ok(SoundBuffer{ inner: Arc::new(buffer), gain: gain, duration: duration })
    }

    pub fn play_event(&mut self, sound_event: SoundEvent, loan: Option<SoundSourceLoan>) -> SoundEventResult<SoundSourceLoan> {
        if let Some(l) = loan {
            if let Some(mut s) = self.sources.for_loan(l) {
                // we have a loan, just apply the event
                s.assign_event(sound_event, l.event_id)?;
                return Ok(l)
            }
        } 
        
        if let Some(buffers) = self.buffers.get(&sound_event.name) {
            // sound is already loaded
            return if let Some((ref mut source, loan)) = self.sources.loan_next_free_static() {
//                 println!("we have a sound event {:?} and now a loan {:?}", sound_event, loan);
                if let Some(buffer) = self.rng.choose(buffers) {
                    source.inner.set_buffer(buffer.inner.clone())?;
                    source.assign_event(sound_event, loan.event_id)?;
                    source.inner.play().map_err(SoundEventError::SoundProviderError)?;
                    Ok(loan)
                } else {
                    Err(SoundEventError::NoSounds)
                }
            } else {
                Err(SoundEventError::NoFreeStaticSource)
            }
        }

        let full_paths = self.full_sound_paths(&sound_event.name)?;

        // ok we need to load/stream it
        let combined_load = load_combined(&full_paths, self.stream_above_file_size)?;

        // we need to call out here ...
        match combined_load {
            LoadedSound::Static(sounds) => {
                let mut buffers = Vec::new();
                for sound in sounds {
                    let buffer = self.buffer_sound(sound, 1.0)?;
                    buffers.push(buffer);
                }

                let sound_event_name = sound_event.name.clone();

                let result = if let Some((source, loan)) = self.sources.loan_next_free_static() {
                    if let Some(buffer) = self.rng.choose(&buffers) {
                        try!(source.inner.set_buffer(buffer.inner.clone()));
                        try!(source.assign_event(sound_event, loan.event_id));
                        try!(source.inner.play());
                        Ok(loan)
                    } else {
                        Err(SoundEventError::NoSounds)
                    }
                } else {
                    Err(SoundEventError::NoFreeStaticSource)
                };

                self.buffers.insert(sound_event_name, buffers);

                result
            },
            LoadedSound::Streaming(ogg_stream_reader) => {
                return if let Some((source, loan)) = self.sources.loan_next_free_streaming() {
                    source.stream_reader = Some((ogg_stream_reader, full_paths[0].clone()));

                    try!(source.ensure_buffers_queued(self.context, self.stream_buffer_duration));
                    try!(source.assign_event(sound_event, loan.event_id));
                    try!(source.inner.play());

                    Ok(loan)
                } else {
                    Err(SoundEventError::NoFreeStreamingSource)
                };
            },
        }
    }

    pub fn ensure_buffers_queued(&mut self) -> PreloadResult<()> {
        for source in self.sources.streaming.iter_mut() {
            if source.current_binding.is_some() {
                match source.ensure_buffers_queued(self.context, self.stream_buffer_duration) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("received error while buffering streaming sources {:?}", err);
                        source.clean()?;
                    },

                }
            }
        }
        Ok(())
    }
}
