#![allow(dead_code)]

#[macro_use]
extern crate howl;

extern crate alto;
extern crate cgmath;

use alto::Alto;

use howl::{Listener, SoundEvent, Vec3f, HashMap};
use howl::engine::SoundWorker;
use howl::engine::SoundEngineUpdate::*;
use cgmath::Zero;

#[cfg(target_os = "windows")] 
const OPENAL_PATH: &'static str = "./native/windows/OpenAL64.dll";
#[cfg(target_os = "macos")]
const OPENAL_PATH: &'static str = "./native/mac/openal.dylib";

fn main() {
    let worker = SoundWorker::create(OPENAL_PATH.into(), "examples/engine/resources".into(), "ogg".into(), 1_000_000, 5.0);

    let listener = Listener::default();

    let sound_event = SoundEvent {
        name: "teleport".into(),
        position: Vec3f::zero(),
        gain: 1.0,
        pitch: 1.5,
        attenuation:1.0,
        loop_sound: false,
    };
    let sound_event_b = SoundEvent {
        name: "water".into(),
        position: Vec3f::zero(),
        gain: 1.0,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    };

    worker.send(Preload(vec![("teleport".into(), 1.0), ("water".into(), 1.0)])).unwrap();

    worker.send(Render { master_gain: 1.0, sounds:vec![sound_event, sound_event_b], persistent_sounds: hashmap!["music".into() => find_me_sound(1.0)], listener: listener }).unwrap();

    std::thread::sleep(std::time::Duration::new(3, 0));

    worker.send(Render { master_gain: 1.0, sounds:Vec::new(), persistent_sounds: hashmap!["music".into() => find_me_sound(0.3)], listener: listener }).unwrap();

    std::thread::sleep(std::time::Duration::new(3, 0));

    worker.send(Render { master_gain: 1.0, sounds:Vec::new(), persistent_sounds: hashmap!["music".into() => find_me_sound(0.3)], listener: listener }).unwrap();

    std::thread::sleep(std::time::Duration::new(3, 0));    

    worker.shutdown_and_wait();

    std::thread::sleep(std::time::Duration::new(1, 0));
}

fn find_me_sound(gain:f32) -> SoundEvent {
    SoundEvent {
        name: "come.and.find.me".into(),
        position: Vec3f::zero(),
        gain: gain,
        pitch: 1.0,
        attenuation:1.0,
        loop_sound: false,
    }
}