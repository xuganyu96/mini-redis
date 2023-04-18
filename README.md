Unknown


# Client
## client object API
`Client` is the struct through which client-side application interfaces with the database and hides the underlying abstract such as `Connection`, `Frame`, and `Command`.

- [ ] `Client::connect` takes an address and return a client if the connection can be successfully established
- [ ] `Client::set` takes a key and a value, sends a `SET <key> <val>` command to the server, then returns the response from the server, which is usually empty
- [ ] `Client::get` takes a key, sends a `GET <key>` command to the server, then returns the response from the server, which might be empty if the key is a miss
- [ ] `Client::pop` takes a key, sends a `POP <key>` command to the server, then returns a response from the server, which is either the value of the popped key or an error message

## Connect to server
First import `myredis::Client` struct. Call `Client::connect` to establish a connection, then return a client object.

```rust
let client = Client::connect("<host>:<port>").await;
```

The `connect` method might fail because the underlying TCP listener cannot be established, so the `connect` method should return a `Result` enum that can surface the error thrown when calling `tokio::net::TcpListener::bind`. **What should the `Err` variant hold???**

If the connection is successfully established, then the client is returned in the `Ok` variant.

### Set/Get/Pop
Each unit of communication with the server always begins with client sending a command, so the client object implements high-level methods that correspond to the set of Redis commands that I want to support, which at this moment there are three: `SET` for setting a key, `GET` for getting a key, and `POP` for deleting a key

For each command that the client sends, the server will respond, sometimes with a status (e.g. if the client send a `SET` command) and sometimes with additional data (e.g. if the client sends a `GET` command). This means that client object's "command" calls expect to return some kind of response.

So we will implement three functions: `get`, `set`, and `pop`. All three of them return a `Result`, with the `Ok` variant holding the complete response from the server and the `Err` variant holding errors for when no valid response (either no response or incomplete response) is received.

```rust
// .get, .set, and .pop all return Result<..., ...> that unwrap once to get
// the response from the server
let resp = client.get(key).await;
```

* `set`  
Given a `key: String` and a `val: String`, send a `SET <key> <val>` command to the server, then return the response `Result<(), Box<dyn Error>>`. The `Ok` variant indicates that the value has been set
* `get`  
Given a `key: &str`, send a `GET <key>` command to the server. The `Ok` variant of the response holds an `Option<String>` where the `None` variant indicates that there is no associated value to the key sent.
* `pop`  
Given a `key: &str`, send a `POP <key>` command to the server. The `Ok` variant of the response holds a `Result<String, ...>` where the `Ok` variant holds the value associated with the key popped, and the `Err` variant holds any error that the server returns (e.g. for when there is no keey to pop)

# Server

# Shared layers of abstraction
There are several layers of abstraction that are shared between server and clients, most of which are related to serialization and deserialization:

* `Bytes`, as a wrapper around a byte array `Vec<u8>`
* `Frame`, which is parsed from `Bytes`, corresponds with the low-level data types of REdis Serialization Protocol (RESP)
    * Simple string
    * Error
    * Bulk string (including NULL)
    * Integer
    * Array
* `Command` is used by client to send commands and by server to parse commands from frames
* `Connection`, which wraps around a TCP stream (what comes out of `listener.accept?`)

Client establishes a connection: it holds a TCP socket wrapped inside a `Connection` object.

* User calls `client.set(key, val)`
    * `client` calls `Command::new_set(key, val)` which returns a `cmd`
    * `client` calls `connection.emit_cmd(cmd)`
        * `Command` first translates itself into the correct `Frame`
        * `Frame` is translated into `Bytes`
        * `Connection` calls `socket.try_write` to write the bytes
    * `client` calls `connection.read_response()`, which returns the frame that the server returns
    * `client` parses the response frame into the correct return value for this call
