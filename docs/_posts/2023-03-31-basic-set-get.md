---
layout: post
title:  "Basic get and set"
date:   2023-03-31 18:00:00 -0700
---

In this section we will realize the most basic Redis functions "set" and "get", which works like this:

```bash
redis set "key" "val"
redis get "key"  # returns "val"
```

# The client application
The client application will be a wrapper around the `mini_redis::client` struct: it parses the CLI arguments and invoke the appropriate functions of the `client` struct.

The main command is the client program itself, which for now will take two optional arguments `host` and `port`. There are also two sub-commands `get`, which takes a positional argument `key`, and `set`, which takes two positional arguments `key` and `val`.

Since each sub-command has its own arguments to parse, we will write them as individual parsers:

```rust
use clap::Parser;  // cargo add clap --features derive

#[derive(Debug,Parser)]
struct Set {
    key: String,
    val: String,
}

#[derive(Debug,Parser)]
struct Get {
    key: String,
}
```

The "choice of sub-command" needs to be a enum with the appropriate attribute, then the enum type is used for the main parser:

```rust
use clap::{ Parser, Subcommand };

#[derive(Debug,Subcommand)]
enum SubArgs {
    Set(Set),
    Get(Get),
}

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    subcmd: SubArgs,

    #[arg()]
    host: Option<String>,

    #[arg()]
    port: Option<usize>,
}
```

The default values for `host` and `port` will be handled at parsing:

```rust
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let host = args.host.unwrap_or("127.0.0.1".to_string());
    let port = args.host.unwrap_or(6379);
    let addr = format!("{host}:{port}");
}
```

Here is an interesting quirk about help text:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::try_parse()?;
    // ...
}
```

If I write argument parsing using `Result` and the question oeprator, then calling `--help` flag will result in the following output instead of the proper help text

```
Error: ErrorInner { kind: DisplayHelp, context: FlatMap { keys: [], values: [] }, message: Some(Formatted(StyledStr("A simple redis client\n\n\u{1b}[1m\u{1b}[4mUsage:\u{1b}[0m \u{1b}[1mclient\u{1b}[0m [HOST] [PORT] <COMMAND>\n\n\u{1b}[1m\u{1b}[4mCommands\u{1b}[0m\u{1b}[1m\u{1b}[4m:\u{1b}[0m\n  \u{1b}[1mset\u{1b}[0m   Set a key-value pair\n  \u{1b}[1mget\u{1b}[0m   Get the value of the input key if available\n  \u{1b}[1mhelp\u{1b}[0m  Print this message or the help of the given subcommand(s)\n\n\u{1b}[1m\u{1b}[4mArguments:\u{1b}[0m\n  [HOST]  Defaults to \"127.0.0.1\"\n  [PORT]  Defaults to 6379\n\n\u{1b}[1m\u{1b}[4mOptions:\u{1b}[0m\n  \u{1b}[1m-h\u{1b}[0m, \u{1b}[1m--help\u{1b}[0m  Print help\n"))), source: None, help_flag: Some("--help"), color_when: Auto, color_help_when: Auto, backtrace: None }
```

This raises some questions about how `--help` flag is processed but they are out of scope of this post so we will just move on.

The remainder of the application will be largely identical to the "Hello, world" example shown in the first post:

```rust
#[tokio::main]
async fn main() {
    // ... parsing arguments ...
    let mut client = client::connect(&addr).await.unwrap();

    match args.subcmd {
        SubArgs::Set(set) => {
            client.set(&set.key, set.val.into()).await.unwrap();
        },
        SubArgs::Get(get) => {
            if let Some(val) = client.get(&get.key).await.unwrap() {
                println!("{}", String::from_utf8(val.to_vec()).unwrap());
            }
        }
    }
}
```

Something to note is that `mini_redis::client`'s `get` and `set` both take `Bytes` for `value`'s type. Long story short, `Bytes` is a wrapper around `Vec<u8>` that provides a more polished interface for working with byte arrays. For now it's okay to simply call `val.into()` and let the implementation of `Bytes` take care of the conversion from `String` into `Bytes`.

To quickly validate the client, launch a container:

```bash
docker run \
  --rm \
  -p 6379:6379 \
  redis:latest
```

Then run the client:

```bash
cargo run --bin client -- set hello world
cargo run --bin client -- get hello
cargo run --bin client -- set hello mom
cargo run --bin client -- get hello
```

# The server

## Concurrent hash map
The core of the logic is a `HashMap<T, U>`, for which we are interested in two methods: `insert(key, val)` and `get(key)`. In a synchronous context, the usage is straightforward:

```rust
use std::collections::HashMap;

fn main() {
    let mut db: HashMap<String, String> = HashMap::new();
    db.insert("hello".to_string(), "world".to_string());
    println!("{:?}", db.get("hello"));
}
```

In an asynchronous context, however, we have to use a mutex to ensure thread-safety. We will also have to use a reference-counted pointer to comply with the rules of the borrow checker:

```rust
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;

type AsyncHashMap<T, U> = Arc<Mutex<HashMap<T, U>>;
```

To modify the underlying hash map, the thread first needs to obtain a lock on the mutex. Calling `Mutex::lock` returns a `Result<MutexGuard>`, where the `MutexGuard` implements the `DerefMut` trait and can thus be used as if it is a mutable reference:

```rust
fn process(
    //  ... other arguments ...
    db: AsyncHashMap<String, Bytes>,
) {
    // ...
    db.lock().unwrap()  // => MutexGuard
        .insert(key, value);   // => can call insert through DerefMut
    
    // ...
    let val = db.lock().unwrap()
        .get(key);
}
```

The `Arc` smart pointer works just like `Rc`: we can use `clone` to create numerous references on the same mutex:

```rust
#[tokio::main]
async fn main() {
    let db: AsyncHashMap<String, Bytes> = Arc::new(Mutex::new(HashMap::new()));

    // ...
    process(..., db.clone())  // enforce borrow checker at runtime instead of
                              // compile time
}

async fn process(
    // ...
    db: AsyncHashMap<String, Bytes>,
) {
    // ...
}
```
