# howl
Opinionated Game Jam Sound Engine in Rust

# TODO
- Streaming looping
- A stategy for recoverable errors, IO/Load errors must be recoverable. Clean the source if it's streaming etc. but don't panic.
- StreamingSoundSource.ensure_buffers_current is the worst function I've ever seen.

Errors are current absurdly verbose and largely reproduce a stack trace because I'm building this in W10 + MSYS and backtraces are broken on this platform.