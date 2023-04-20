//! Shared layers of abstraction: Bytes, Frame, Command, Connection, Client
use std::error::Error;
use tokio::net::{ TcpListener, TcpStream };

type MyResult<T> = Result<T, Box<dyn Error>>;

/// A client provides high-level methods for sending commands to and receiving
/// commands from the server.
pub struct Client {
    listener: TcpListener,
    // ... other states ...
}

impl Client {
    /// Connect to the server specified at the input address, or return any
    /// error while attempting the connect
    pub async fn connect(addr: &str) -> MyResult<Self> {
        let listener = TcpListener::bind(addr).await?;
        return Ok(Self { listener });
    }

    /// Send a "SET key val" command to the server. Return the Ok variant if
    /// the server returns a OK.
    pub async fn set(&mut self, key: String, val: String) -> MyResult<()> {
        todo!();
    }

    /// Send a "GET key" command to the server. Return the Ok variant if the
    /// server returns some valid results.
    ///
    /// If the key has a match in the server, then the associated value is
    /// returned in the "Some" variant. If the key has no match, then the
    /// "None" variant is returned.
    pub async fn get(&mut self, key: String) -> MyResult<Option<String>> {
        todo!();
    }

    /// Send a "POP key" command to the server. Return the Ok variant if the
    /// server returns some valid results.
    ///
    /// If the key has a match in the server and the key is successfully
    /// deleted, then the associated value is returned. If the key has no
    /// match, or other things went wrong during the deletion, then the
    /// error message is returned.
    pub async fn pop(&mut self, key: String) -> MyResult<Result<String, String>> {
        todo!();
    }
}

/// The Command enum provides abstraction over Frames
enum Command {
    Set{ key: String, val: String },
    Get{ key: String, },
    Pop{ key: String, },
}

impl Command {
    /// Create a new Set command
    fn set(key: String, val: String) -> Self {
        return Self::Set{ key, val };
    }

    /// Create a new Get command
    fn get(key: String) -> Self {
        return Self::Get{ key };
    }

    /// Create a new Pop command
    fn pop(key: String) -> Self {
        return Self::Pop{ key };
    }

    /// Convert a command into the appropriate Frame
    fn to_frame(&self) -> Frame {
        todo!();
    }
}

/// The various RESP data types
enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Vec<u8>),
    Array(Vec<Frame>),
}

impl Frame {
    /// Serialize a frame into a byte array for transmission over the network
    fn as_bytes(&self) -> Bytes {
        todo!();
    }

    /// Read the input byte array and check if there is a valid frame inside.
    /// If yes, return the parsed frame in the "Some" variant, else return
    /// None
    fn parse(bytes: &Bytes) -> Option<Frame> {
        todo!();
    }
}

/// A wrapper around a byte array with a cursor
struct Bytes {
    bytes: Vec<u8>,
    cursor: usize,
}

/// A wrapper around a TCP socket (TcpStream) for writing byte stream into
/// Bytes and for parsing Bytes into frames
struct Connection {
    socket: TcpStream,
}

impl Connection {
    /// Instantiate a new connection
    fn new(socket: TcpStream) -> Self {
        return Self { socket };
    }

    /// Read bytes from the TcpStream, then parse it. If there is a valid
    /// Frame in the bytes read, then return it. Else return None.
    fn read_frame() -> Option<Frame> {
        todo!();
    }

    /// Convert the input frame into bytes, then write into the socket
    fn write_frame() {
        todo!();
    }

    /// Convert the command into the correct Frame, then pass into write_frame
    fn emit_command(&mut self, cmd: Command) {
    }
}
