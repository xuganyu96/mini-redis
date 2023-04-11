//! A sample Rust client that demonstrates the usage patterns of tokio::sync::mpsc
//! and tokio::sync::oneshot
use bytes::Bytes;
use mini_redis;
use tokio::sync::{
    mpsc,
    oneshot::{self, Sender},
};

type Response<T> = mini_redis::Result<T>;

#[derive(Debug)]
enum Command {
    Set {
        key: String,
        val: Bytes,
        callback: Sender<Response<()>>,
    },
    Get {
        key: String,
        callback: Sender<Response<Option<Bytes>>>,
    },
}

impl Command {
    fn new_set(key: String, val: String, tx: Sender<Response<()>>) -> Self {
        return Self::Set {
            key,
            val: val.into(),
            callback: tx,
        };
    }

    fn new_get(key: String, tx: Sender<Response<Option<Bytes>>>) -> Self {
        return Self::Get { key, callback: tx };
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32);

    let manager = tokio::spawn(async move {
        let mut client = mini_redis::client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Set { key, val, callback } => {
                    let result = client.set(&key, val).await;
                    callback.send(result).unwrap();
                }
                Command::Get { key, callback } => {
                    let result = client.get(&key).await;
                    callback.send(result).unwrap();
                }
            }
        }
    });

    let task = tokio::spawn(async move {
        let (otx, orx) = oneshot::channel();
        tx.send(Command::new_set("hello".into(), "world".into(), otx))
            .await
            .unwrap();
        let result = orx.await.unwrap();
        println!("{:?}", result);

        let (otx, orx) = oneshot::channel();
        tx.send(Command::new_get("hello".into(), otx))
            .await
            .unwrap();
        let result = orx.await.unwrap();
        println!("{:?}", result);

        let (otx, orx) = oneshot::channel();
        tx.send(Command::new_set("hello".into(), "mom".into(), otx))
            .await
            .unwrap();
        let result = orx.await.unwrap();
        println!("{:?}", result);

        let (otx, orx) = oneshot::channel();
        tx.send(Command::new_get("hello".into(), otx))
            .await
            .unwrap();
        let result = orx.await.unwrap();
        println!("{:?}", result);
    });

    manager.await.unwrap();
    task.await.unwrap();
}
