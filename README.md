# mini-redis
Follow along tokio.rs and building mini-redis

## Scope of project

- `lib.rs`
    - `Bytes`, for wrapping around `Vec<u8>` and providing buffered reading and writing
    - `Connection`
        - `read_frame`
        - `write_frame`
        - `parse_frame`
    - `Frame::Simple, Error, Integer, Bulk, Array`
        - `parse`
    - `Command::Get, Set, Del`
    - `Client`
        - `get`
        - `set`
        - `pop`
    - `DB`: the concurrent HashMap
- `bin`
    - A client CLI application
    - A server CLI application
    - Both with options `-h/--host` and `-p/--port`
    - The server CLI application offers multiple flavors of concurrency (including synchronous server)
- Maybe:
    - Benchmarking?