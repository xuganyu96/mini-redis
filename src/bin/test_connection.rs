//! Test the Connection struct
//! To perform the test, launch an echo server with:
//! cargo run --bin echo server -- -p 6379
//!
//! This program will send a frame and check if it can read it back
use bytes::Bytes;
use redis::Connection;
use redis::Frame;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let socket = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let mut conn = Connection::new(socket);

    let frame = Frame::Bulk(Bytes::from("Hello, world!"));
    conn.write_frame(&frame).await.unwrap();
    assert_eq!(conn.read_frame().await.unwrap(), Some(frame));
}
