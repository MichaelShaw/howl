pub mod engine;
pub mod load;
pub mod context;
pub mod source;

extern crate cgmath;
extern crate alto;
extern crate ogg;
extern crate lewton;
extern crate fnv;
extern crate time;

use fnv::FnvHasher;
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};
use std::hash::BuildHasherDefault;

pub type Vec3f = cgmath::Vector3<f32>;

// blend speed for persistent sounds, in, out?

pub type SoundName = String;

pub type SoundEventId = u64; 

pub type Gain = f32;

pub type DistanceModel = alto::DistanceModel;

#[derive(Clone, Debug)]
pub struct SoundEvent {
    pub name: String,
    pub position: Vec3f,
    pub gain: f32,
    pub pitch: f32,
    pub attenuation: f32, // unsure if this should be bool for relative, or an optional rolloff factor (within the context distance model)
    pub loop_sound: bool,
}

pub type Listener = self::context::Listener;

pub type HowlResult<T> = errors::Result<T>;
pub type HowlError = errors::Error;

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        errors {
            NoFreeStaticSource
            NoFreeStreamingSource
            TooManyChannels
            FileDoesntExist(whatever: ::std::path::PathBuf)
        }

        foreign_links {
            IO(::std::io::Error);
            Vorbis(::lewton::VorbisError);
            Alto(::alto::AltoError);
        }
    }
}

pub type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;
pub type HashSet<V> = StdHashSet<V, BuildHasherDefault<FnvHasher>>;

#[macro_export]
macro_rules! hashset {
    ($($val: expr ),*) => {{
         let mut set = HashSet::default();
         $( set.insert( $val); )*
         set
    }}
}

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = HashMap::default();
         $( map.insert($key, $val); )*
         map
    }}
}
