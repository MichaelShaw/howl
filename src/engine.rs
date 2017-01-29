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
    Clear, // unbind all sources, destroy all buffers
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

    pub fn process(&mut self, context: &mut SoundContext, update:SoundEngineUpdate) -> HowlResult<()> {
        use self::SoundEngineUpdate::*;
        match update {
            Preload(sounds) => {
                for &(ref sound_name, gain) in &sounds {
                    println!("preload {:?} gain {:?}", sound_name, gain);
                    try!(context.load_sound(sound_name, gain));
                }
            },
            DistanceModel(model) => {
                try!(context.set_distace_model(model))
            },
            Render { master_gain, sounds, persistent_sounds, listener } => {
                println!("::::: RENDER :::::");
                try!(context.sources.clean());
                try!(context.ensure_buffers_current());
                println!("post ensure");
                if context.master_gain != master_gain {
                    println!("updating master gain to {:?}", master_gain);
                    try!(context.set_gain(master_gain));
                }
                if context.listener != listener {
                    println!("updating listener!");
                    try!(context.set_listener(listener));
                }
                for sound_event in sounds {
                    println!("play event {:?}", sound_event);
                    try!(context.play_event(sound_event, None));
                    println!("post event");
                }
                
                for (name, sound_event) in persistent_sounds {
                    println!("pre persistent");
                    let old_loan = self.loans.remove(&name);
                    let new_loan = try!(context.play_event(sound_event, old_loan));
                    self.loans.insert(name, new_loan);
                    println!("post persistent");
                }
                println!("post render");

                ()   
            },
            Clear => {
                try!(context.purge());
                ()
            },
        };
        Ok(())
    }
}
