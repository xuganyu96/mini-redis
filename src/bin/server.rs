//! Asynchronous Redis server.
//!
//! There is a single listener on the port, but every connection will move
//! into a new task using tokio::spawn
use std::collections::HashMap;
use tokio;
use tokio::net::{ TcpListener, TcpStream };
use mini_redis::{ Connection, Frame, Command };
use std::sync::{ Arc, Mutex };
use bytes::Bytes;

type AsyncHashMap = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: AsyncHashMap = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let db = db.clone();
        tokio::spawn(async move {
            shared_process(stream, db.clone()).await;
        });
    }
}

async fn shared_process(stream: TcpStream, db: AsyncHashMap) {
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
