//! Redis server
use bytes::Bytes;
use redis::Command;
use redis::Connection;
use redis::Frame;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            let connection = Connection::new(socket);
            let _ = process(connection, addr).await;
        });
    }
}

async fn process(mut connection: Connection, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    loop {
        let frame = connection.read_frame().await?;
        match frame {
            None => {
                println!("Disconnected from {addr:?}");
                return Ok(());
            }
            Some(frame) => {
                let cmd = Command::parse_command(&frame);
                match cmd {
                    None => {
                        connection
                            .write_frame(&Frame::Error("Illegal command".into()))
                            .await?;
                    }
                    Some(Command::Set { key: _, val: _ }) => {
                        connection.write_frame(&Frame::Simple("OK".into())).await?;
                    }
                    Some(Command::Get { key: _ }) => {
                        connection
                            .write_frame(&Frame::Bulk(Bytes::from("Hello")))
                            .await?;
                    }
                    Some(Command::Del { key: _ }) => {
                        connection.write_frame(&Frame::Integer(0)).await?;
                    }
                }
            }
        };
    }
}
