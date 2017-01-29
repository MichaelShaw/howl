use alto;
use alto::{Context, Buffer, SourceTrait};
use alto::{Mono, Stereo};

use std::sync::Arc;
use std::path::{PathBuf};

use super::{Gain, SoundEvent, SoundName, DistanceModel};
use super::load::{load_combined, LoadedSound, load_ogg};
use super::source::{Sources, SoundSource, StreamingSoundSource, SoundSourceLoan};

use Vec3f;
use HashMap;
use HowlResult;

use cgmath::Zero;
use errors::*;

pub struct SoundContext<'d> {
    pub context: &'d Context<'d>,
    pub path: String,
    pub extension: String,
    pub sources: Sources<'d>,
    pub buffers: HashMap<SoundName, SoundBuffer<'d>>,
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



#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Listener {
    pub position: Vec3f,
    pub velocity: Vec3f,
    pub orientation_up: Vec3f,
    pub orientation_forward: Vec3f,
}

impl Listener {
    pub fn default() -> Listener {
        Listener {
            position: Vec3f::zero(),
            velocity: Vec3f::zero(),
            orientation_up: Vec3f::new(0.0, 1.0, 0.0),
            orientation_forward: Vec3f::new(0.0, 0.0, -1.0),
        }
    }
}

pub fn create_sound_context<'d>(context: &'d Context<'d>, path:&str, extension: &str, stream_above_file_size: u64, stream_buffer_duration: f32) -> SoundContext<'d> {
    // we should probably create our sources here
    SoundContext {
        context: context,
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
    pub fn set_gain(&mut self, gain: Gain) -> HowlResult<()> {
        self.context.set_gain(gain).chain_err(||"SoundContext attempting to set gain")?;
        self.master_gain = gain;

        Ok(())
    }

    pub fn create(&mut self, static_count: usize, streaming_count: usize) -> HowlResult<()> {
        for n in 0..static_count {
            let source = self.context.new_static_source().chain_err(||format!("Attempting to create {:?} static source", n))?;
            self.sources.sources.push(SoundSource { inner: source, current_binding: None});
        }
        for n in 0..streaming_count {
            let source = self.context.new_streaming_source().chain_err(||format!("Attempting to create {:?} streaming source", n))?;
            self.sources.streaming.push(StreamingSoundSource { inner: source, stream_reader: None, current_binding: None });
        }
        Ok(())
    }

    pub fn set_listener(&mut self, listener: Listener) -> HowlResult<()> {
        self.context.set_position(listener.position).chain_err(||"Attempting to set position")?;
        self.context.set_velocity(listener.velocity).chain_err(||"Attempting to set velocity")?;
        self.context.set_orientation::<[f32; 3]>((listener.orientation_forward.into(), listener.orientation_up.into())).chain_err(||"Attempting to set velocity")?;

        self.listener = listener;
        
        Ok(())
    }

     pub fn purge(&mut self) -> HowlResult<()> {
        self.sources.purge().chain_err(|| "Context Purging sources")?;
        self.buffers.clear();
        Ok(())
    }

    pub fn full_path(&self, name: &str) -> PathBuf {
        PathBuf::from(format!("{}/{}.{}", &self.path, name, &self.extension))
    }


    pub fn set_distace_model(&mut self, distance_model: DistanceModel) -> HowlResult<()> {
        self.context.set_distance_model(distance_model).chain_err(||"Context setting distance model")?;
        self.distance_model = distance_model;
        Ok(())
    }

    // just convenience
    pub fn stop(&mut self, loan:SoundSourceLoan) -> HowlResult<()> {
        if let Some(ref mut source) = self.sources.for_loan(loan) {
            source.stop().chain_err(||format!("Attempting to stop source for {:?}", loan))?;
        }
        Ok(())
    }

    pub fn load_sound(&mut self, sound_name: &str, gain: Gain) -> HowlResult<()> {
        let path = self.full_path(sound_name);
        let sound = load_ogg(&path).chain_err(|| format!("attempting to load path {:?}", path))?;
        let mut buffer = try!(self.context.new_buffer());
        let duration = sound.duration();
        if sound.channels == 1 {
            try!(buffer.set_data::<Mono<i16>, _>(sound.data, sound.sample_rate as i32));
        } else if sound.channels == 2 {
            try!(buffer.set_data::<Stereo<i16>, _>(sound.data, sound.sample_rate as i32));
        } else {
            bail!(ErrorKind::TooManyChannels);
        }

        let arc_buffer = Arc::new(buffer);
        self.buffers.insert(sound_name.into(), SoundBuffer{ inner: arc_buffer, gain: gain, duration: duration });

        Ok(())
    }

    pub fn play_event(&mut self, sound_event: SoundEvent, loan: Option<SoundSourceLoan>) -> HowlResult<SoundSourceLoan> {
        if let Some(l) = loan {
            if let Some(mut s) = self.sources.for_loan(l) {
                // we have a loan, just apply the event
                s.assign_event(sound_event, l.event_id).chain_err(||"Attempting to assign event to source")?;
                return Ok(l)
            }
        } 
        
        if let Some(buffer) = self.buffers.get(&sound_event.name) {
            // sound is loaded
            return if let Some((ref mut source, loan)) = self.sources.loan_next_free_static() {
                // println!("we have a sound event {:?} and now a loan {:?}", sound_event, loan);
                // and there's a free source
                source.inner.set_buffer(buffer.inner.clone())?;
                source.assign_event(sound_event, loan.event_id)?;
                source.inner.play().chain_err(||"Playing source")?;
     
                Ok(loan)
            } else {
                Err(ErrorKind::NoFreeStaticSource.into())
            }
        }

        // ok we need to load/stream it
        let full_path = self.full_path(&sound_event.name);
        let combined_load = load_combined(&full_path, self.stream_above_file_size)?;

        // we need to call out here ...
        match combined_load {
            LoadedSound::Static(sound) => {
                let mut buffer = try!(self.context.new_buffer());
                let duration = sound.duration();
                if sound.channels == 1 {
                    try!(buffer.set_data::<Mono<i16>, _>(sound.data, sound.sample_rate as i32));
                } else if sound.channels == 2 {
                    try!(buffer.set_data::<Stereo<i16>, _>(sound.data, sound.sample_rate as i32));
                } else {
                    bail!(ErrorKind::TooManyChannels);
                }

                let arc_buffer = Arc::new(buffer);
        
                let sound_name = sound_event.name.clone();
                
                let result = if let Some((source, loan)) = self.sources.loan_next_free_static() {
                    try!(source.inner.set_buffer(arc_buffer.clone()));
                    try!(source.assign_event(sound_event, loan.event_id));
                    try!(source.inner.play());
                    Ok(loan)
                } else {
                    Err(ErrorKind::NoFreeStaticSource.into())
                };
                self.buffers.insert(sound_name, SoundBuffer{ inner: arc_buffer, gain: 1.0, duration: duration });
                result
            },
            LoadedSound::Streaming(ogg_stream_reader) => {
                return if let Some((source, loan)) = self.sources.loan_next_free_streaming() {
                    source.stream_reader = Some((ogg_stream_reader, full_path));

                    try!(source.ensure_buffers_queued(self.context, self.stream_buffer_duration));
                    try!(source.assign_event(sound_event, loan.event_id));
                    try!(source.inner.play());

                    Ok(loan)
                } else {
                    Err(ErrorKind::NoFreeStreamingSource.into())
                };
            },
        }
    }

    pub fn ensure_buffers_queued(&mut self) -> HowlResult<()> {
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