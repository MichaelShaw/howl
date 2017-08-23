# howl
Opinionated Game Jam Sound Engine in Rust

Productivity through hotloading and declarative syntax over effiency.

# Usage
See 'engine' example.

Use the worker module to construct a SoundWorker (runs on a seperate thread) through SoundWorker::create, and send it SoundEngineUpdate's, call shutdown_and_wait() when you're done.

Sound worker will swallow/recover from load errors (missing files, ogg read errs, file type errors) and capacity errors (e.g. no free sources), but will halt immediately upon an OpenAL error.

Upon noticing a file change to it's resource directory it will purge all buffers/music to allow hot loading (will improve this to reload specific buffers at some point).

# TODO
- Proper Streaming looping (it'll loop ... but without prebuffering, so it will likely stutter) ... it can't recreate it's stream
- StreamingSoundSource.ensure_buffers_current is the worst function I've ever seen/written.
- Beef up persistent sounds. Add simple blending (we currently have none, you can do it manually, but it'd be nice for music and looping sounds etc.)
- Make looping of non-persistent sounds impossible. This is currently a footgun/landmine.

We currently load before we loan a source, which means after the loan only SoundProviderErrors can happen ... which we halt on ... so in theory no source will become in a stuck state. For streaming we cleanup if there's been a load error.