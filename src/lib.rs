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

use std::path::PathBuf;
use std::io;

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

pub type HowlResult<T> = Result<T, HowlError>;

#[derive(Debug)]
pub enum HowlError {
	IO(io::Error),
	FileDoesntExist(PathBuf),
	Vorbis(lewton::VorbisError),
    Alto(alto::AltoError),
    TooManyChannels,
    NoSound(String),
    NoFreeSource(bool), // bool is for streaming
}


impl From<lewton::VorbisError> for HowlError {
    fn from(val: lewton::VorbisError) -> HowlError {
        HowlError::Vorbis(val)
    }
}

impl From<io::Error> for HowlError {
    fn from(val: io::Error) -> HowlError {
        HowlError::IO(val)
    }
}

impl From<alto::AltoError> for HowlError {
    fn from(val: alto::AltoError) -> HowlError {
        HowlError::Alto(val)
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
