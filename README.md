# howl
Opinionated Game Jam Sound Engine in Rust

# Usage
Use the engine moduleto costruct a SoundWorker (runs on a seperate thread) through SoundWorker::create, and send it SoundEngineUpdate's, shutdown_and_wait() when you're done.

# TODO
- Hotloading, notifier etc.
- Streaming looping (in theory it'll sort of automatically loop once it ends and cleans up)
- A stategy for recoverable errors, IO/Load errors must be recoverable. Clean the source if it's streaming etc. but don't panic.
- StreamingSoundSource.ensure_buffers_current is the worst function I've ever seen/written.
- Remove error chain? We need our own well typed heirarchy, or better discipline. chain_err makes types awkward when I want to discriminate on them.

Errors are current absurdly verbose and largely reproduce a stack trace because I'm building this in W10 + MSYS and backtraces are broken on this platform.