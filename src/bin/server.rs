use bytes::Bytes;
use redis::{Command, Connection, Frame, MyResult};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

struct DB<T, U> {
    db: Mutex<HashMap<T, U>>,
}

impl<T: Eq + Hash, U: Clone> DB<T, U> {
    fn insert(&self, key: T, val: U) -> Option<U> {
        let mut lock = self.db.lock().unwrap();
        return lock.insert(key, val);
    }

    fn get(&self, key: &T) -> Option<U> {
        let lock = self.db.lock().unwrap();
        return lock.get(key).cloned();
    }

    fn remove(&self, key: &T) -> Option<U> {
        let mut lock = self.db.lock().unwrap();
        return lock.remove(&key);
    }

    fn new() -> Self {
        let db = Mutex::new(HashMap::new());
        return Self { db };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    let db: Arc<DB<Bytes, Bytes>> = Arc::new(DB::new());
    loop {
        let (socket, _addr) = listener.accept().await?;
        let connection = Connection::new(socket);
        let db_copy = Arc::clone(&db);

        tokio::spawn(async move {
            let _ = process(connection, db_copy).await;
        });
    }
}

async fn process(mut connection: Connection, db: Arc<DB<Bytes, Bytes>>) -> MyResult<()> {
    loop {
        let frame = connection.read_frame().await?;
        match frame {
            None => {
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
                            connection.write_frame(&Frame::Bulk(val)).await?;
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
