---
layout: post
title:  "Shared states, cont'"
date:   2023-04-05 20:42:00 -0700
---

The second half of the ["shared states"](https://tokio.rs/tokio/tutorial/shared-state) section mainly covered two topics:

1. Contention and ways of addressing them
2. Holding mutex across `await`

# Contention and ways of addressing them
Recall that a mutex uses certain synchronization primitives provided by the operating systems and the underlying hardware to guard a piece of memory against non-atomic writes. This means that while some thread is holding the lock on the mutex, other threads calling the `lock` function on the mutex will be waiting, and thus we have **contention**. When there are many threads waiting on the same resource, contention can cause the program to slow down significantly.

## Use a single-threaded runtime
By default the async runtime of tokio is multithreaded, meaning the task scheduler will spawn multiple threads when multiple `tokio::spawn` calls are made. However, for when the throughput is not that high, we can use a single-threaded runtime by specifying the runtime flavor at the `tokio::main` attribute:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // all tasks will be executed on the same thread as the main function
}
```

## Shard the data structure
Another strategy for addressing contention is to shard the shared state. There are production-ready structs such as [dashmap](https://docs.rs/dashmap/latest/dashmap/). We will implement a "poor-man's sharded hash map" in a later section to illustrate this point alonside some later points

# Holding a lock across await
Recall that each task that is passed into `tokio::spawn` must be `Send` because when a task calls `await` on an async function, the task scheduler will yield control from the caller's thread to the called function's thread. This is problematic because the mutex guard from `std::sync` does not implement `Send`, and so the compiler will not compile the code. For example, if we write the `process` function in the following manner:

```rust
async fn process(stream: TcpStream, db: Arc<Mutex<HashMap<String, Bytes>>>) {
    let mut connection = Connection::new(stream);
    while let some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let mut lock = db.lock().unwrap():
        let response = match cmd {
            // ... match ...
        };
        connection.write_frame(&response).await.unwrap();
    }
}
```

The compiler will report the following error:

> future cannot be sent between threads safely  
within `impl Future<Output = ()>`, the trait `Send` is not implemented for `std::sync::MutexGuard<...>`

This section mentions three ways to bypass this problem:

## Explicitly drop the lock
The most straightforward solution is to use scope to ensure that the lock is dropped before any `await` is called. We can refactor the implementation above to put the lock in the match block:

```rust
async fn process(stream: TcpStream, db: Arc<Mutex<HashMap<String, Bytes>>>) {
    let mut connection = Connection::new(stream);
    while let some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let response = match cmd {
            Command::Set(cmd) => {
                let lock = db.lock();
                // ...
            },
            Command::Get(cmd) => {
                let lock = db.lock();
                // ...
            },
            _ => // ...
        }  // the lock will be dropped when the match block exits

        connection.write_frame(&response).await.unwrap();
    }
}
```

## Synchronous wrapper
This solution is an extension of the first solution in that it also addresses "holding lock across await" by dropping the lock, but this time the drop is more gracefully done by wrapping the lock within a synchronous function call.

Here is a rough idea:

```rust
struct DB<T, U> {
    db: Mutex<HashMap<T, U>>,
}

impl<T: Eq + Hash, U: Clone> DB<T, U> {
    fn new() -> Self {
        let db = Mutex::new(HashMap::new());
        return Self { db };
    }

    fn get(&self, key: &T) -> Option<U> {
        // Locks are obtained and dropped within a synchronous function call
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
```

## Use tokio's Mutex
A third solution is to use `tokio::sync::Mutex` instead of `std::sync::Mutex`, the former of which does implement `Send` and is a drop-in substitute for `std::sync::Mutex`.

# Poor man's sharded hashmap
```rust
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
```