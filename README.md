# howl
Opinionated Game Jam Sound Engine in Rust

# Usage
Use the engine moduleto costruct a SoundWorker (runs on a seperate thread) through SoundWorker::create, and send it SoundEngineUpdate's, shutdown_and_wait() when you're done.

# TODO
- Streaming looping (in theory it'll sort of automatically loop once it ends and cleans up)
- StreamingSoundSource.ensure_buffers_current is the worst function I've ever seen/written.
- Beef up persistent sounds. Add simple blending (we currently have none, you can do it manually, but it'd be nice for music and looping sounds etc.)
- Make looping of non-persistent sounds impossible. This is currently a footgun/landmine.

# TODO ... Error Handling
- A stategy for recoverable errors, IO/Load errors must be recoverable. Clean the source if it's streaming etc. but don't panic.
- Remove error chain? We need our own well typed heirarchy, or better discipline. chain_err makes types awkward when I want to discriminate on them.

