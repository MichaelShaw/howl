use {HashMap, HowlResult};

use time;

use super::{DistanceModel, SoundEvent, Gain, SoundName};
use super::context::{Listener, SoundContext};
use super::source::SoundSourceLoan;

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

    pub fn process(&mut self, context: &mut SoundContext, update:SoundEngineUpdate) -> HowlResult<bool> {
        use self::SoundEngineUpdate::*;
        let halt = match update {
            Preload(sounds) => {
                for &(ref sound_name, gain) in &sounds {
                    match context.load_sound(sound_name, gain) {
                        Ok(_) => (),
                        Err(err) => {
                            println!("Sound Worker failed to preload {:?} err -> {:?}", sound_name, err);
                            ()
                        },
                    }
                }
                false
            },
            DistanceModel(model) => {
                try!(context.set_distace_model(model));
                false
            },
            Render { master_gain, sounds, persistent_sounds, listener } => {
                try!(context.sources.check_bindings());
                try!(context.ensure_buffers_queued());
                if context.master_gain != master_gain {
                    try!(context.set_gain(master_gain));
                }
                if context.listener != listener {
                    try!(context.set_listener(listener));
                }
                for sound_event in sounds {
                    match context.play_event(sound_event.clone(), None) {
                        Ok(_) => (),
                        Err(err) => {
                            println!("Sound Worker had problem playing sound_event {:?} err -> {:?}", sound_event, err);
                            ()
                        },
                    }
                }
                
                for (name, sound_event) in persistent_sounds {
                    let old_loan = self.loans.remove(&name);
                    let new_loan = try!(context.play_event(sound_event, old_loan));
                    self.loans.insert(name, new_loan);
                }

                false  
            },
            Clear => {
                try!(context.purge());
                false
            },
            Stop => {
                try!(context.purge());
                true
            }
        };
        Ok(halt)
        
    }
}
