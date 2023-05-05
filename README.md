# Server
The server side application has two main parts:

- Parsing the command from the TCP stream and writing response to the TCP socket
- Making the appropriate state mutation, most likely on a HashMap

The majority fo the server logic has already been implemented once while reading through the mini-redis tutorial, so there is not much to cover on the server side.

# Interesting bug:

```rust
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        handler().await.unwrap();
    });
    return Ok(());
}

async fn handler() -> Result<(), Box<dyn Error>> {
    loop {
        let result: Result<(), Box<dyn Error>> = Ok(());
        match result? {
            () => {
                hello().await;
            }
        }
    }
}

async fn hello() {
    println!("Hello");
}
```
