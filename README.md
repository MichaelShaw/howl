# howl
Opinionated Game Jam Sound Engine in Rust

# Usage
See 'engine' example.

Use the worker module to construct a SoundWorker (runs on a seperate thread) through SoundWorker::create, and send it SoundEngineUpdate's, call shutdown_and_wait() when you're done.

Sound worker will swallow/recover from load errors (missing files, ogg read errs, file type errors) and capacity errors (e.g. no free sources), but will halt immediately upon an OpenAL error.

Upon noticing a file change to it's resource directory it will purge all buffers/music to allow hot loading (will improve this to reload specific buffers at some point).

# TODO
- Streaming looping (in theory it'll sort of automatically loop once it ends and cleans up)
- StreamingSoundSource.ensure_buffers_current is the worst function I've ever seen/written.
- Beef up persistent sounds. Add simple blending (we currently have none, you can do it manually, but it'd be nice for music and looping sounds etc.)
- Make looping of non-persistent sounds impossible. This is currently a footgun/landmine.
