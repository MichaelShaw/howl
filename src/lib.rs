pub mod engine;
pub mod load;
pub mod context;
pub mod source;
pub mod worker;

extern crate cgmath;
extern crate alto;
// extern crate ogg;
extern crate lewton;
extern crate time;
extern crate notify;
extern crate aphid;
extern crate rand;

use cgmath::Zero;

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

pub type LoadResult<T> = Result<T, errors::LoadError>;
pub type PreloadResult<T> = Result<T, errors::PreloadError>;
pub type SoundEventResult<T> = Result<T, errors::SoundEventError>;
pub type SoundProviderResult<T> = Result<T, alto::AltoError>;
pub type WorkerResult<T> = Result<T, alto::AltoError>;

use std::fs;
use std::path::{PathBuf, Path};

pub fn read_directory_paths(path:&Path) -> PreloadResult<Vec<PathBuf>> {
    use errors::{LoadError, LoadErrorReason};
    let mut paths : Vec<PathBuf> = Vec::new();

    for entry in try!(fs::read_dir(path).map_err(|io| LoadError {path: path.to_path_buf(), reason: LoadErrorReason::FileReadError(io) })) {
        let entry = try!(entry.map_err(|io| LoadError {path: path.to_path_buf(), reason: LoadErrorReason::FileReadError(io) }));
        let file_path = entry.path().to_path_buf();
        paths.push(file_path);
    }

    Ok(paths)
}

pub mod errors {
    use alto;
    use std::path::PathBuf;
    use lewton;
    use std::io;

    #[derive(Debug)]
    pub enum PreloadError {
        LoadError(LoadError),
        SoundProviderError(alto::AltoError), // this is a dupe at this point ... hrm
    }

    impl From<LoadError> for PreloadError {
        fn from(val: LoadError) -> PreloadError {
            PreloadError::LoadError(val)
        }
    }

    impl From<alto::AltoError> for PreloadError {
        fn from(val: alto::AltoError) -> PreloadError {
            PreloadError::SoundProviderError(val)
        }
    }

    #[derive(Debug)]
    pub struct LoadError {
        pub path: PathBuf,
        pub reason: LoadErrorReason,
    }

    #[derive(Debug)]
    pub enum LoadErrorReason {
        FileDoesntExist,
        FileReadError(io::Error),
        ReadOggError(lewton::VorbisError),
        TooManyChannels,
    }

    #[derive(Debug)]
    pub enum SoundEventError {
        LoadSoundError(LoadError), // recoverable
        SoundProviderError(alto::AltoError),
        NoFreeStaticSource,
        NoFreeStreamingSource,
        NoSounds,
    }

    impl From<LoadError> for SoundEventError {
        fn from(val: LoadError) -> SoundEventError {
            SoundEventError::LoadSoundError(val)
        }
    }

    impl From<alto::AltoError> for SoundEventError {
        fn from(val: alto::AltoError) -> SoundEventError {
            SoundEventError::SoundProviderError(val)
        }
    }

    impl From<PreloadError> for SoundEventError {
        fn from(val: PreloadError) -> SoundEventError {
            use self::SoundEventError::*;
            match val {
                PreloadError::LoadError(le) => LoadSoundError(le),
                PreloadError::SoundProviderError(ae) => SoundProviderError(ae), 
            }
        }
    }
}

// - OpenALError (unrecoverable)
// - SoundEventError (Load | OpenAL | NoFreeStreamingSource | NoFreeStaticSource)
