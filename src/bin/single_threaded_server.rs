//! A synchronous Redis server: every request is blocking
use bytes::Bytes;
use mini_redis::Command;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use std::hash::Hash;

struct DB<T, U> {
    db: Mutex<HashMap<T, U>>,
}

impl<T: Eq + Hash, U: Clone> DB<T, U> {
    fn new() -> Self {
        let db = Mutex::new(HashMap::new());
        return Self { db };
    }

    fn get(&self, key: &T) -> Option<U> {
        return match self.db.lock() {
            Ok(lock) => {
                match lock.get(key) {
                    Some(val) => Some(val.clone()),
                    None => None,
                }
            },
            _ => None,
        }
    }

    fn set(&self, key: T, val: U) {
        if let Ok(mut lock) = self.db.lock() {
            lock.insert(key, val);
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: Arc<DB<String, Bytes>> = Arc::new(DB::new());

    while let Ok((stream, _addr)) = listener.accept().await {
        let db = db.clone();
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(stream: TcpStream, db: Arc<DB<String, Bytes>>) {
    let mut connection = Connection::new(stream);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let response = match cmd {
            Command::Set(cmd) => {
                db.set(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => match db.get(&cmd.key().to_string()) {
                Some(val) => Frame::Bulk(val.clone().into()),
                None => Frame::Null,
            },
            _ => unimplemented!("{:?} not implemented!", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
