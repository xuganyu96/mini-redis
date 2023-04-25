//! Shared layers of abstraction: Bytes, Frame, Command, Connection, Client
use bytes::{ Buf, Bytes };
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

type MyResult<T> = Result<T, Box<dyn Error>>;

const CRLF: &str = "\r\n";

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
    Set { key: String, val: String },
    Get { key: String },
    Pop { key: String },
}

impl Command {
    /// Create a new Set command
    fn set(key: String, val: String) -> Self {
        return Self::Set { key, val };
    }

    /// Create a new Get command
    fn get(key: String) -> Self {
        return Self::Get { key };
    }

    /// Create a new Pop command
    fn pop(key: String) -> Self {
        return Self::Pop { key };
    }

    /// Convert a command into the appropriate Frame
    fn to_frame(&self) -> Frame {
        todo!();
    }
}

/// The various RESP data types. The data types are explained here:
/// https://redis.io/docs/reference/protocol-spec/
#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Vec<u8>),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    /// Serialize a frame into a byte array for transmission over the network
    fn serialize(&self) -> Bytes {
        return match self {
            Self::Simple(_) => self._serialize_simple_string(),
            Self::Error(_) => self._serialize_error(),
            Self::Integer(_) => self._serialize_integer(),
            Self::Bulk(_) => self._serialize_bulk_string(),
            Self::Null => self._serialize_null(),
            Self::Array(_) => self._serialize_array(),
        };
    }

    /// Serialize a simple string to a byte array.
    fn _serialize_simple_string(&self) -> Bytes {
        // "+<content>\r\n"
        if let Self::Simple(s) = self {
            let data = format!("+{}{}", s, CRLF);
            return Bytes::copy_from_slice(&data.as_bytes());
        }
        unreachable!("Self is not Frame::Simple");
    }

    /// Serialize an error to a byte array
    fn _serialize_error(&self) -> Bytes {
        // "-<err>\r\n"
        if let Self::Error(s) = self {
            let data = format!("-{}{}", s, CRLF);
            return Bytes::copy_from_slice(&data.as_bytes());
        }
        unreachable!("Self is not Frame::Error");
    }

    /// Serialize a u64 integer to a byte array
    fn _serialize_integer(&self) -> Bytes {
        // ":<integer>\r\n"
        if let Self::Integer(n) = self {
            let data = format!(":{n}{CRLF}");
            return Bytes::copy_from_slice(&data.as_bytes());
        }
        unreachable!("Self is not Frame::Integer!");
    }

    /// Serialize a bulk string, including the empty string, but not including
    /// Frame::Null
    fn _serialize_bulk_string(&self) -> Bytes {
        // "$<len><CRLF><data><CRLF>"
        if let Self::Bulk(arr) = self {
            let len = arr.len();
            let buf = vec![
                b"$".to_vec(),
                format!("{len}").as_bytes().to_vec(),
                CRLF.as_bytes().to_vec(),
                arr.to_vec(),
                CRLF.as_bytes().to_vec(),
            ]
            .concat();

            return Bytes::copy_from_slice(&buf);
        }
        unreachable!("Self is not Frame::Bulk");
    }

    /// Serialize a Null frame, which is a Bulk frame with a length of -1
    fn _serialize_null(&self) -> Bytes {
        // "$-1<CRLF>"
        if let Self::Null = self {
            let buf = format!("$-1{CRLF}");
            return Bytes::copy_from_slice(&buf.as_bytes());
        }
        unreachable!("Self is not Frame::Null");
    }

    /// Serialize an Array frame by recursively calling "serialize" on its
    /// elements
    fn _serialize_array(&self) -> Bytes {
        if let Self::Array(v) = self {
            let nelems = v.len();
            let prefix = Bytes::from(format!("*{nelems}{CRLF}").as_bytes().to_vec());
            let elems: Bytes = v
                .iter()
                .map(|frame| frame.serialize())
                .collect::<Vec<Bytes>>()
                .concat()
                .into();
            return vec![prefix, elems].concat().into();
        }
        unreachable!("Self is not Frame::Array");
    }

    /// Read the input byte array and check if there is a valid frame inside.
    /// If yes, return the parsed frame in the "Some" variant, else return
    /// None
    fn parse(bytes: &Bytes) -> Option<Frame> {
        if bytes.starts_with(b"+") {
            if let Some(msg) = Self::_parse_binary_safe_string(&mut bytes.slice(1..)) {
                return Some(Frame::Simple(msg));
            }
            return None;
        } else if bytes.starts_with(b"-") {
            if let Some(msg) = Self::_parse_binary_safe_string(&mut bytes.slice(1..)) {
                return Some(Frame::Error(msg));
            }
            return None;
        } else if bytes.starts_with(b":") {
            if let Some(num) = Self::_parse_binary_safe_string(&mut bytes.slice(1..)) {
                if let Ok(num) = num.parse::<i64>() {
                    return Some(Frame::Integer(num));
                }
            }
            return None;
        } else if bytes.starts_with(b"$") {
        } else if bytes.starts_with(b"*") {
        }
        return None;
    }

    /// Given some bytes that are assumed to be binary safe, extract the
    /// string between the start of the bytes and the first CRLF. If the bytes
    /// do not contain CRLF, return None
    fn _parse_binary_safe_string(bytes: &mut Bytes) -> Option<String> {
        let mut msg = vec![];
        
        while bytes.has_remaining() && !bytes.starts_with(CRLF.as_bytes()) {
            msg.push(bytes.get_u8());
        }

        if bytes.has_remaining() {
            return Some(String::from_utf8(msg).unwrap());
        }
        return None;
    }
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
    fn emit_command(&mut self, cmd: Command) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_serialization() {
        let simple = Frame::Simple("OK".into());
        let expected: Vec<u8> = b"+OK\r\n".to_vec();
        assert_eq!(
            simple.serialize(),
            Bytes::copy_from_slice(&expected)
        );
    }

    #[test]
    fn test_integer_serialization() {
        let num = Frame::Integer(0);
        let expected = b":0\r\n";
        assert_eq!(num.serialize(), Bytes::copy_from_slice(expected));
    }

    #[test]
    fn test_bulk_string_serialization() {
        let bulk = Frame::Bulk(vec![b'6', b'9', b'4', b'2', b'0']);
        let expected = b"$5\r\n69420\r\n";
        assert_eq!(
            bulk.serialize(),
            Bytes::copy_from_slice(expected)
        );
    }

    #[test]
    fn test_null_serialization() {
        let null = Frame::Null;
        assert_eq!(null.serialize(), Bytes::from(b"$-1\r\n".to_vec()));
    }

    #[test]
    fn test_array_serialization() {
        let set = Frame::Simple("SET".into());
        let key = Frame::Bulk("foo".as_bytes().to_vec());
        let val = Frame::Bulk("bar".as_bytes().to_vec());
        let cmd = Frame::Array(vec![set, key, val]);
        assert_eq!(
            cmd.serialize(),
            Bytes::from(b"*3\r\n+SET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_vec())
        );
    }

    #[test]
    fn test_simple_string_deserialization() {
        assert_eq!(
            Frame::parse(&Bytes::from("+SET\r\n")),
            Some(Frame::Simple("SET".into())),
        );

        assert_eq!(
            Frame::parse(&Bytes::from("+SET\r\n+++++")),
            Some(Frame::Simple("SET".into())),
        );

        assert_eq!(
            Frame::parse(&Bytes::from("+SET\r")),
            None,
        );

        assert_eq!(
            Frame::parse(&Bytes::from("+\r\n")),
            Some(Frame::Simple("".into())),
        );
    }

    #[test]
    fn test_error_deserialization() {
        assert_eq!(
            Frame::parse(&Bytes::from("-Key not found\r\n")),
            Some(Frame::Error("Key not found".into())),
        );

        assert_eq!(
            Frame::parse(&Bytes::from("-Key not found\r\n-----")),
            Some(Frame::Error("Key not found".into())),
        );

        assert_eq!(
            Frame::parse(&Bytes::from("-Key not found\r")),
            None,
        );

        assert_eq!(
            Frame::parse(&Bytes::from("-\r\n")),
            Some(Frame::Error("".into())),
        );
    }

    #[test]
    fn test_integer_deserialization() {
        assert_eq!(
            Frame::parse(&Bytes::from(":0\r\n")),
            Some(Frame::Integer(0)),
        );

        assert_eq!(
            Frame::parse(&Bytes::from(":1\r\n")),
            Some(Frame::Integer(1)),
        );

        assert_eq!(
            Frame::parse(&Bytes::from(":-1\r\n")),
            Some(Frame::Integer(-1)),
        );
        
        assert_eq!(
            Frame::parse(&Bytes::from(":9223372036854775807\r\n")),
            Some(Frame::Integer(9223372036854775807)),
        );
        
        assert_eq!(
            Frame::parse(&Bytes::from(":-9223372036854775808\r\n")),
            Some(Frame::Integer(-9223372036854775808)),
        );
        
        assert_eq!(
            Frame::parse(&Bytes::from(":9223372036854775808\r\n")),
            None,
        );
        
    }

    #[test]
    fn test_bulk_string_deserialization() {
    }

    #[test]
    fn test_null_frame_deserialization() {
    }

    #[test]
    fn test_array_deserialization() {
    }
}
