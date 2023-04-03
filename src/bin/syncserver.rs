//! A synchronous Redis server: every request is blocking
use mini_redis::{Connection, Frame};
use mini_redis::Command;
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use bytes::Bytes;
use std::sync::{Arc, Mutex};

type AsyncHashMap<T, U> = Arc<Mutex<HashMap<T, U>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: AsyncHashMap<String, Bytes> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let db = db.clone();
        let (stream, _addr) = listener.accept().await.unwrap();
        process(stream, db).await;
    }
}

async fn process(stream: TcpStream, db: AsyncHashMap<String, Bytes>) {
    let mut connection = Connection::new(stream);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let response = match cmd {
            Command::Set(cmd) => {
                db.lock().unwrap()
                    .insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            },
            Command::Get(cmd) => {
                match db.lock().unwrap().get(cmd.key()) {
                    Some(val) => Frame::Bulk(val.clone().into()),
                    None => Frame::Null,
                }
            },
            _ => unimplemented!("{:?} not implemented!", cmd),
        };
        
        connection.write_frame(&response).await.unwrap();
    }
}
