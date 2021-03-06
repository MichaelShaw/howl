#![allow(dead_code)]

extern crate howl;

extern crate alto;

extern crate rand;

#[macro_use]
extern crate aphid;

use aphid::HashMap;

use howl::{Listener, SoundEvent, Vec3, VEC3_ZERO};
use howl::worker::SoundWorker;
use howl::engine::SoundEngineUpdate::*;
use howl::engine::SoundRender;



#[cfg(target_os = "windows")] 
const OPENAL_PATH: &'static str = "./native/windows/OpenAL64.dll";
#[cfg(target_os = "macos")]
const OPENAL_PATH: &'static str = "./native/mac/openal.dylib";

fn main() {
    let rand = rand::XorShiftRng::new_unseeded();
    let worker = SoundWorker::create(OPENAL_PATH.into(), "examples/engine/resources".into(), "ogg".into(), rand, 1_000_000, 5.0);

    let listener = Listener::default();

    let sound_event = SoundEvent {
        name: "teleport".into(),
        position: VEC3_ZERO,
        gain: 1.0,
        pitch: 1.5,
        attenuation:1.0,
        loop_sound: false,
    };
    let sound_event_b = SoundEvent {
        name: "water".into(),
        position: VEC3_ZERO,
        gain: 1.0,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    };

    worker.send(Preload(vec![("teleport".into(), 1.0), ("water".into(), 1.0)])).unwrap();

    worker.send(Render(SoundRender { master_gain: 1.0, sounds:vec![sound_event.clone(), sound_event_b.clone()], persistent_sounds: hashmap!["music".into() => find_me_sound(1.0)], listener: listener })).unwrap();

    std::thread::sleep(std::time::Duration::new(3, 0));

    worker.send(Render(SoundRender { master_gain: 1.0, sounds:vec![sound_event_b.clone()], persistent_sounds: hashmap!["music".into() => find_me_sound(0.3)], listener: listener })).unwrap();

    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::new(2, 0));

        worker.send(Render(SoundRender { master_gain: 1.0, sounds:vec![sound_event.clone()], persistent_sounds: hashmap!["music".into() => find_me_sound(0.3)], listener: listener })).unwrap();
    }

    std::thread::sleep(std::time::Duration::new(3, 0));    

    worker.shutdown_and_wait();
}

fn find_me_sound(gain:f32) -> SoundEvent {
    SoundEvent {
        name: "come.and.find.me".into(),
        position: VEC3_ZERO,
        gain: gain,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    }
}