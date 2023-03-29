---
layout: post
title:  "Hello, Tokio"
date:   2023-03-27 21:10:27 -0700
---

Add the `tokio` crate and the `mini-redis` crate to the project:

```
cargo add tokio@1.27.0 --features "full"
cargo add mini-redis@0.4
```

Create the source code file for the mini-redis client program at `src/bin/client.rs`:

```rust
use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("hello", "world".into()).await?;
    let result = client.get("hello").await?;
    if let Some(val) = result {
        println!("hello: {:?}", val);
    }
    return Ok(());
}
```

Add the following section to `Cargo.toml` (although maybe cargo can actually automatically detect the content of `src/bin`, so maybe this is not really necessary)

```toml
[[bin]]
name = "client"
path = "src/bin/client.rs"
```

The tutorial advised installing `mini-redis` using `cargo install mini-redis` and running the server instance using the `mini-redis-server` binary, but I tried using the official Redis Docker image and the code still works:

```
docker run --rm -p 6378:6378 redis:latest
cargo run --bin client
```

A few things to note:

1. In standard Rust, the `await` keyword must be used in an `async` function or code block
2. The `main` function cannot be `async`, but the `#[tokio::main]` macro can be used to convert an async `main` function into an asynchronous block in a synchronous `main` function. Note that `tokio` does not need to be imported for this macro to work.
3. The body of the `async` function is not executed until `await`

This is a synchronous "Hello, world"
```rust
fn main() {
    println!("Hello, world!");
}
```

Say we move the "world!" part into an async function:

```rust
async say_world() {
    println!("world!");
}

fn main() {
    let handle = say_world();
    print!("Hello, ");
    handle.await;
}
```

This doesn't work because of reason 1, but adding `async` to the `main` function won't solve the problem because of reason 2. The fully functional asynchronous "Hello, world" looks like this:

```rust
async fn say_world() {
    println!("world!");
}

#[tokio::main]
async fn main() {
    let handle = say_world();
    print!("Hello, ");
    handle.await;
}
```

Note that despite having called `say_world` before calling `print!("Hello, ")`, the text "world!" is still printed AFTER "Hello, " because of reason 3.