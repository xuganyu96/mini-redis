//! Test the Connection struct
//! To perform the test, launch an echo server with:
//! cargo run --bin echo server -- -p 6379
//!
//! This program will send a frame and check if it can read it back
use bytes::Bytes;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use redis::Connection;
use redis::Frame;

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let frame = Frame::Bulk(Bytes::from("Hello, world!"));
    socket.write_all(&frame.serialize()).await.unwrap();
    let mut conn = Connection::new(socket);
    assert_eq!(conn.read_frame().await.unwrap(), Some(frame));
}
