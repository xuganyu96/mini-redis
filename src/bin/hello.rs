//! Run a Redis server using Docker:
//!
//! ```bash
//! docker run --rm -p 6379:6379 redis:latest
//! ```
//!
//! Then open a TCP connection using netcat:
//!
//! ```bash
//! nc -v 127.0.0.1 6379
//! ```
use bytes::BytesMut;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:6379").await?;
    println!("Connected");
    if socket.writable().await.is_ok() {
        let buffer = "*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"; // "SET foo bar"
        socket.write_all(&buffer.as_bytes()).await?;
        println!("Buffer written");
    }
    socket.readable().await?;
    let mut buf = BytesMut::new();
    let nbytes = socket.read_buf(&mut buf).await?;
    println!("{nbytes} bytes of buffer read");
    println!("{:?}", buf);
    return Ok(());
}
