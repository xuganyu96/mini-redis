use bytes::Bytes;
use redis::{Command, Connection, Frame};
use std::collections::HashMap;
use std::error::Error;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    let mut db: HashMap<Bytes, Bytes> = HashMap::new();
    loop {
        let (socket, addr) = listener.accept().await?;
        let mut connection = Connection::new(socket);

        println!("Connected to {addr:?}");
        loop {
            let frame = connection.read_frame().await?;
            match frame {
                None => {
                    // read_frame returns None iff the socket read 0 bytes,
                    // signifying a closed connection
                    println!("{addr:?} disconnected");
                    break;
                }
                Some(frame) => {
                    let cmd = Command::parse_command(&frame);
                    match cmd {
                        None => {
                            connection
                                .write_frame(&Frame::Error("Illegal command".into()))
                                .await?;
                        }
                        Some(Command::Set { key, val }) => {
                            db.insert(key, val);
                            connection.write_frame(&Frame::Simple("OK".into())).await?;
                        }
                        Some(Command::Get { key }) => match db.get(&key) {
                            None => {
                                connection
                                    .write_frame(&Frame::Error("Key not found".into()))
                                    .await?;
                            }
                            Some(val) => {
                                connection.write_frame(&Frame::Bulk(val.clone())).await?;
                            }
                        },
                        Some(Command::Del { key }) => match db.remove(&key) {
                            None => {
                                connection.write_frame(&Frame::Integer(0)).await?;
                            }
                            Some(_) => {
                                connection.write_frame(&Frame::Integer(1)).await?;
                            }
                        },
                    }
                }
            }
        }
    }
}
