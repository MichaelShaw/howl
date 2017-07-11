use time;

use {Listener, DistanceModel, SoundEvent, Gain, SoundName, SoundProviderResult};
use super::context::{SoundContext};
use super::source::SoundSourceLoan;
use super::errors::*;
use aphid::HashMap;

#[derive(Debug, Clone)]
pub enum SoundEngineUpdate {
    Preload(Vec<(SoundName, Gain)>), // load buffers
    DistanceModel(DistanceModel),
    Render { master_gain: f32, sounds:Vec<SoundEvent>, persistent_sounds:HashMap<String, SoundEvent>, listener: Listener },
    Clear, // unbind all sources, destroy all buffers,
    Stop,
}

// we need our state of what's already persisted, loans etc.

pub struct SoundEngine {
    // some measure of time
    // some notion of existing sounds
    pub last_render_time: u64,
    pub loans : HashMap<String, SoundSourceLoan>,
}

impl SoundEngine {
    pub fn new() -> SoundEngine {
        SoundEngine {
            last_render_time: time::precise_time_ns(),
            loans: HashMap::default(),
        }
    }

    pub fn process(&mut self, context: &mut SoundContext, update:SoundEngineUpdate) -> SoundProviderResult<bool> { // book is over clean shutdown
        use self::SoundEngineUpdate::*;
        let should_continue = match update {
            Preload(sounds) => {
                for (sound_name, gain) in sounds {
                    match context.preload(&sound_name, gain) {
                        Ok(buffers) => (),
                        Err(err) => {
                            println!("Sound Worker failed to preload {:?} err -> {:?}", sound_name, err);
                            ()
                        },
                    }
                }
                true
            },
            DistanceModel(model) => {
                context.set_distace_model(model)?;
                true
            },
            Render { master_gain, sounds, persistent_sounds, listener } => {
                try!(context.sources.check_bindings());
                match context.ensure_buffers_queued() {
                    Ok(_) => (),
                    Err(PreloadError::LoadError(le)) => println!("Sound worker received load error while ensuring buffers are queued {:?}", le),
                    Err(PreloadError::SoundProviderError(sp)) => return Err(sp), 
                }
                if context.master_gain != master_gain {
                    try!(context.set_gain(master_gain));
                }
                if context.listener != listener {
                    try!(context.set_listener(listener));
                }

                for sound_event in sounds {
                    match context.play_event(sound_event.clone(), None) {
                        Ok(_) => (),
                        Err(SoundEventError::SoundProviderError(sp)) => return Err(sp),
                        Err(err) => println!("Sound Worker had problem playing sound_event {:?} err -> {:?}", sound_event, err),
                    }
                }
                
                for (name, sound_event) in persistent_sounds {
                    let old_loan = self.loans.remove(&name);
                    match context.play_event(sound_event.clone(), old_loan) {
                        Ok(new_loan) => {
                            self.loans.insert(name, new_loan);        
                        },
                        Err(SoundEventError::SoundProviderError(sp)) => return Err(sp),
                        Err(err) => println!("Sound Worker had problem playing sound_event {:?} err -> {:?}", sound_event, err),
                    }
                }

                true  
            },
            Clear => {
                try!(context.purge());
                true
            },
            Stop => {
                try!(context.purge());
                false
            }
        };
        Ok(should_continue)
    }
}
