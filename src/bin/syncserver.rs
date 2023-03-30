//! A synchronous Redis server: every request is blocking
use mini_redis::{Connection, Frame};
use mini_redis::Command;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> MyResult<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Established connection");

    for _ in 0..10 {
        let (stream, _addr) = listener.accept().await?;
        process(stream).await;
    }

    return Ok(());
}

async fn process(stream: TcpStream) {
    let mut db: HashMap<String, String> = HashMap::new();
    let mut connection = Connection::new(stream);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        println!("{:?}", cmd);
        let resp = match cmd {
            Command::Set(cmd) => {
                let key = cmd.key();
                let val = String::from_utf8_lossy(&cmd.value().to_vec()).to_string();
                db.insert(key.to_string(), val);
                Frame::Simple("OK".to_string())
            },
            Command::Get(cmd) => {
                let val = db.get(cmd.key());
                match val {
                    Some(val) => Frame::Bulk(val.clone().into()),
                    None => Frame::Null,
                }
            },
            _ => unreachable!("Command not implemented yet"),
        };
        connection.write_frame(&resp).await.unwrap();
    }
}
