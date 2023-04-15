Unknown
- [ ] What should `Client::connect`'s Err variant hold?
- [ ] How should responses form the server be abstracted?

Tentative assumption:

# Client
## Connect to server
First import `myredis::Client` struct. Call `Client::connect` to establish a connection, then return a client object.

```rust
let client = Client::connect("<host>:<port>").await;
```

The `connect` method might fail because the underlying TCP listener cannot be established, so the `connect` method should return a `Result` enum that can surface the error thrown when calling `tokio::net::TcpListener::bind`. **What should the `Err` variant hold???**

If the connection is successfully established, then the client is returned in the `Ok` variant.

## client object API
Each unit of communication with the server always begins with client sending a command, so the client object implements high-level methods that correspond to the set of Redis commands that I want to support, which at this moment there are three: `SET` for setting a key, `GET` for getting a key, and `POP` for deleting a key

For each command that the client sends, the server will respond, sometimes with a status (e.g. if the client send a `SET` command) and sometimes with additional data (e.g. if the client sends a `GET` command). This means that client object's "command" calls expect to return some kind of response.

So we will implement three functions: `get`, `set`, and `pop`. All three of them return a `Result`, with the `Ok` variant holding the complete response from the server and the `Err` variant holding errors for when no valid response (either no response or incomplete response) is received.

```rust
// .get, .set, and .pop all return Result<..., ...> that unwrap once to get
// the response from the server
let resp = client.get(key).await;
```
