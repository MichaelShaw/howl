use {HashMap, HowlResult};

use time;

use alto::Alto;

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


use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SendError;

// 
pub struct SoundWorker {
    send_channel: Sender<SoundEngineUpdate>,
    join_handle: JoinHandle<()>,
}

impl SoundWorker {
    pub fn send(&self, update: SoundEngineUpdate) -> Result<(), SendError<SoundEngineUpdate>> {
        self.send_channel.send(update)
    }

    pub fn shutdown_and_wait(self) {
        println!("sending stop");
        self.send(SoundEngineUpdate::Stop).unwrap();
        println!("joining");
        self.join_handle.join().unwrap();
        println!("thread joined");
    }

    pub fn create(open_al_path: String, resources_path:String, extension:String, streaming_threshold: u64, streaming_buffer_duration: f32) -> SoundWorker {
        let (tx, rx) = channel::<SoundEngineUpdate>();
        let join_handle = thread::spawn(move || {
            let alto = Alto::load(open_al_path).unwrap();
            let dev = alto.open(None).unwrap();
            let ctx = dev.new_context(None).unwrap();
            let mut cb = super::context::create_sound_context(&ctx, &resources_path, &extension, streaming_threshold, streaming_buffer_duration);

            cb.create(32, 4).unwrap();

            let mut engine = SoundEngine::new();
            loop {
                match rx.recv() {
                    Ok(event) => {
                        println!("worker receiving event {:?}", event);
                        match engine.process(&mut cb, event) {
                            Ok(halt) => {
                                if halt {
                                    println!("Sound engine shutting down");
                                    break;
                                }
                            },
                            Err(err) => {
                                println!("Sound engine received unrecoverable error {:?} and is shutting down", err);
                                break;
                            },
                        }
                    },
                    Err(recv_error) => {
                        println!("Sound worker received error {:?}", recv_error);
                        break;
                    },
                }
            }
        });

        SoundWorker {
            send_channel: tx,
            join_handle: join_handle,
        }
    }
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
                    try!(context.load_sound(sound_name, gain));
                }
                false
            },
            DistanceModel(model) => {
                try!(context.set_distace_model(model));
                false
            },
            Render { master_gain, sounds, persistent_sounds, listener } => {
                try!(context.sources.clean());
                try!(context.ensure_buffers_queued());
                if context.master_gain != master_gain {
                    try!(context.set_gain(master_gain));
                }
                if context.listener != listener {
                    try!(context.set_listener(listener));
                }
                for sound_event in sounds {
                    try!(context.play_event(sound_event, None));
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
