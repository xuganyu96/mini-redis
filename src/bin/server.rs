//! Asynchronous Redis server.
//!
//! There is a single listener on the port, but every connection will move
//! into a new task using tokio::spawn
use std::collections::HashMap;
use tokio;
use tokio::net::{ TcpListener, TcpStream };
use mini_redis::{ Connection, Frame, Command };
use std::sync::{ Arc, Mutex };
use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use bytes::Bytes;

struct ShardedHashMap<T, U> {
    shards: Vec<Mutex<HashMap<T, U>>>,
}

impl<T: Hash + Eq, U: Clone> ShardedHashMap<T, U> {
    fn new(n: usize) -> Self {
        let mut shards = vec![];
        for _ in 0..n {
            shards.push(Mutex::new(HashMap::new()));
        }

        return Self { shards };
    }

    fn get(&self, key: &T) -> Option<U> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let shard = self.shards.get(hasher.finish() as usize % self.shards.len());
        if let Some(shard) = shard {
            let lock = shard.lock().unwrap();
            let val = lock.get(key);
            if let None = val {
                return None;
            }
            if let Some(val) = val {
                return Some(val.clone());
            }
        }
        return None;
    }

    fn set(&self, key: T, val: U) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let shard = self.shards.get(hasher.finish() as usize % self.shards.len());
        if let Some(shard) = shard {
            let mut lock = shard.lock().unwrap();
            lock.insert(key, val);
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db: Arc<ShardedHashMap<String, Bytes>> = Arc::new(ShardedHashMap::new(10));

    while let Ok((stream, _)) = listener.accept().await {
        let db = db.clone();
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(stream: TcpStream, db: Arc<ShardedHashMap<String, Bytes>>) {
    let mut connection = Connection::new(stream);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let response = match cmd {
            Command::Set(cmd) => {
                db.set(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            },
            Command::Get(cmd) => {
                match db.get(&cmd.key().to_string()) {
                    Some(val) => Frame::Bulk(val.clone().into()),
                    None => Frame::Null,
                }
            },
            _ => unimplemented!("{:?} not implemented!", cmd),
        };
        
        connection.write_frame(&response).await.unwrap();
    }
}
