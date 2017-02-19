
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SendError;

use alto::Alto;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, RawEvent};

use super::engine::{SoundEngineUpdate, SoundEngine};
    
    // // , Duration::from_secs(0)
    
    



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
        // println!("sending stop");
        self.send(SoundEngineUpdate::Stop).unwrap();
        // println!("joining");
        self.join_handle.join().unwrap();
        // println!("thread joined");
    }

    pub fn create(open_al_path: String, resources_path:String, extension:String, streaming_threshold: u64, streaming_buffer_duration: f32) -> SoundWorker {
        let (tx, rx) = channel::<SoundEngineUpdate>();
        let join_handle = thread::spawn(move || {
            let alto = Alto::load(open_al_path).unwrap();
            let dev = alto.open(None).unwrap();
            let ctx = dev.new_context(None).unwrap();
            let mut cb = super::context::create_sound_context(&ctx, &resources_path, &extension, streaming_threshold, streaming_buffer_duration);

            cb.create(32, 4).unwrap();

            let (notify_tx, notify_rx) = channel::<RawEvent>();
            let mut watcher : RecommendedWatcher = Watcher::new_raw(notify_tx).expect("a watcher");
            watcher.watch(&resources_path, RecursiveMode::Recursive).expect("watching shader vertex path");

            let mut engine = SoundEngine::new();
            loop {
                match rx.recv() {
                    Ok(event) => {
                        // println!("worker receiving event {:?}", event);
 						
 						let mut purge = false;
 						'fs: loop {
	 						match notify_rx.try_recv() {
	                			Ok(RawEvent { path, op:_, cookie:_ }) => {
	                				println!("sound worker noticed path changed -> {:?}", path);
	                				purge = true;
	                			}
	                			Err(_) => {
	                				break 'fs;
	                			}
	                		}
	                	}

	                	if purge {
	                		// at some point we could do smarter purging
	                		println!("sound worker noticed file system changes, purging buffers");
	        				if engine.process(&mut cb, SoundEngineUpdate::Clear).is_err() {
                                break;
                            }
	                	}

                        match engine.process(&mut cb, event) {
                            Ok(true) => (),
                            Ok(false) => {
                                // println!("Sound engine shutting down");
                                break;
                            },
                            Err(err) => {
                                println!("Sound engine received unrecoverable error {:?} and is shutting down", err);
                                break;
                            },
                        }
                    },
                    Err(recv_error) => {
                        println!("Sound worker received error when reading from channel {:?}", recv_error);
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