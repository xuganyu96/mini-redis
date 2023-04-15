---
layout: post
title:  "回声服务器"
date:   2023-04-11 21:23:00 -0700
---

今天写一个回声服务器（echo server）。服务器监听特定的 TCP 端口，将所有收到的数据直接返还。

首先建立 TCP 端口的链接：

```rust
use tokio::io;
use tokio::net::TcpListener;

#[tokio::main]
asyn fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    while let Ok((socket, _addr)) = listener.accept().await {
        tokio::spawn(async {
            //  ... 服务器的核心逻辑 ...
        });
    }
    return Ok(());
}
```

# 拆分读取器和写入器
`tokio::io` 自带 `copy` 函数可以把输入的读取器的数据写入输入的写入器，但是虽然 `socket: TcpStream` 同时满足 `AsyncRead` 和 `AsyncWrite`，Rust 的借查规则不允许 `copy` 同时接纳两个 `socket` 的可变引用。

```rust
asyn fn echo(socket: TcpStream) {
    io::copy(&mut socket, &mut socket);  // will not compile
}
```

对于同时满足 `AsyncRead` 和 `AsyncWrite` 的对象，有以下几个方法将对象拆分成分开的读取器（满足 `AsyncRead`）和写入器（满足 `AsyncWrite`）：

1. 使用 `tokio::io::split`。`split` 的幕后使用 `Arc` 和 `Mutex` 实现拆分，所以拆分以后的读取器和写入器可以分散到不同线程的任务中，但是使用这些同步原语会带来性能上的损失
2. 使用 `TcpStream.split`。这个函数的输入是 `socket` 的不可变引用，所以拆分以后的读取器不可以分散到不同的线程中，但是不会带来性能损失（零成本抽象）
3. 使用 `TcpStream.into_split`。这个函数使用 `Arc` 实现拆分，所以拆分后的读写器可以分散，但是会带来少量性能损失（但是因为不使用 `Mutex` 所以比第一种性能更好）

```rust
async fn echo_with_copy(socket: TcpStream) {
    // let (mut reader, mut writer) = io::split(socket).unwrap();
    let (mut reader, mut writer) = socket.split();
    // let (mut reader, mut writer) = socket.into_split();
    io::copy(&mut reader, &mut writer).await;
}
```

# 手动实现 copy
如果不使用 `tokio::io::copy`，则需要手动分配一个缓冲：在循环中不停地先从端口把数据读进缓存，再把数据从缓存写入。在异步任务中不推荐在堆上分配缓存：如果缓存需要跨过 `await`，tokio 有可能需要把整个缓存转移位置。

```rust
async fn echo_with_buffer(socket: TcpStream) {
    let mut buffer = vec![0; 10];

    loop {
        match socket.read(&mut buffer).await {
            Ok(n) if n == 0 => {  // 端口已经关闭
                return Ok(());
            },
            Ok(n) => {
                socket.write_all(&buffer[..n]).await?;
            },
            Err(e) => {
                eprintln!("echo_server: {e}");
                return Ok(());
            }
        }
    }
}
```