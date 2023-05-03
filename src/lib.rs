//! Shared layers of abstraction: Bytes, Frame, Command, Connection, Client
use bytes::{Buf, Bytes, BytesMut};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

type MyResult<T> = Result<T, Box<dyn Error>>;

const CRLF: &str = "\r\n";

/// A client provides high-level methods for sending commands to and receiving
/// commands from the server.
pub struct Client {
    connection: Connection,
}

impl Client {
    /// Connect to the server specified at the input address, or return any
    /// error while attempting the connect
    pub async fn connect(addr: &str) -> MyResult<Self> {
        let socket = TcpStream::connect(addr).await?;

        return Ok(Self {
            connection: Connection::new(socket),
        });
    }

    /// Send a "SET key val" command to the server. Return the Ok variant if
    /// the server returns a OK.
    pub async fn set(&mut self, key: &str, val: &str) -> MyResult<()> {
        let cmd = Frame::Array(vec![
            Frame::Bulk(Bytes::from("SET")),
            // TODO: unnecessary copy
            Frame::Bulk(Bytes::copy_from_slice(key.as_bytes())),
            Frame::Bulk(Bytes::copy_from_slice(val.as_bytes())),
        ]);
        self.connection.write_frame(&cmd).await?;
        self.connection.read_frame().await?;

        return Ok(());
    }

    /// Send a "GET key" command to the server. Return the Ok variant if the
    /// server returns some valid results.
    ///
    /// If the key has a match in the server, then the associated value is
    /// returned in the "Some" variant. If the key has no match, then the
    /// "None" variant is returned.
    pub async fn get(&mut self, key: &str) -> MyResult<Option<Bytes>> {
        let cmd = Frame::Array(vec![
            Frame::Bulk(Bytes::from("GET")),
            Frame::Bulk(Bytes::copy_from_slice(key.as_bytes())),
        ]);
        self.connection.write_frame(&cmd).await?;
        let resp = self.connection.read_frame().await?;
        if let Some(Frame::Bulk(bytes)) = resp {
            return Ok(Some(bytes));
        }
        return Ok(None);
    }

    /// Send a "DEL key" command to the server. Return the Ok variant if the
    /// server returns some valid results.
    ///
    /// If the key has a match in the server and the key is successfully
    /// deleted, then the associated value is returned. If the key has no
    /// match, or other things went wrong during the deletion, then the
    /// error message is returned.
    pub async fn del(&mut self, key: &str) -> MyResult<Option<i64>> {
        let cmd = Frame::Array(vec![
            Frame::Bulk(Bytes::from("DEL")),
            Frame::Bulk(Bytes::copy_from_slice(key.as_bytes())),
        ]);
        self.connection.write_frame(&cmd).await?;
        let resp = self.connection.read_frame().await?;
        if let Some(Frame::Integer(num)) = resp {
            return Ok(Some(num));
        }
        return Ok(None);
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
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    /// Serialize a frame into a byte array for transmission over the network
    pub fn serialize(&self) -> Bytes {
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
    fn parse(bytes: &mut Bytes) -> Option<Frame> {
        if !bytes.has_remaining() {
            return None;
        }
        match bytes.get_u8() {
            b'+' => {
                if let Some(msg) = Self::_parse_binary_safe_string(bytes) {
                    return Some(Frame::Simple(msg));
                }
            }
            b'-' => {
                if let Some(msg) = Self::_parse_binary_safe_string(bytes) {
                    return Some(Frame::Error(msg));
                }
            }
            b':' => {
                if let Some(num) = Self::_parse_binary_safe_string(bytes) {
                    if let Ok(num) = num.parse::<i64>() {
                        return Some(Frame::Integer(num));
                    }
                }
            }
            b'$' => {
                // Check against Null frame
                if bytes.starts_with(b"-1\r\n") {
                    return Some(Frame::Null);
                }

                // Read until the first CRLF to parse the number of bytes
                if let Some(nbytes) = Self::_parse_binary_safe_string(bytes) {
                    if let Ok(nbytes) = nbytes.parse::<usize>() {
                        // bytes[0..nbytes] should be the content
                        // bytes[nbytes..nbytes+2] should be another CRLF
                        if bytes.remaining() >= nbytes + 2
                            && bytes.slice(nbytes..nbytes + 2).starts_with(CRLF.as_bytes())
                        {
                            let frame = Frame::Bulk(Bytes::from(bytes.slice(0..nbytes)));
                            bytes.advance(nbytes + 2);
                            return Some(frame);
                        }
                    }
                }
            }
            b'*' => {
                // Parsing an array: first obtain the number of elements, then
                // fill a Vector with that number of elements
                if let Some(nelems) = Self::_parse_binary_safe_string(bytes) {
                    if let Ok(nelems) = nelems.parse::<usize>() {
                        let mut elems = vec![];

                        for _ in 0..nelems {
                            if let Some(frame) = Self::parse(bytes) {
                                elems.push(frame);
                            } else {
                                return None;
                            }
                        }

                        return Some(Frame::Array(elems));
                    }
                }
            }
            _ => (),
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
            // CRLF should be consumed, as well
            bytes.advance(CRLF.as_bytes().len());
            if let Ok(msg) = String::from_utf8(msg) {
                return Some(msg);
            }
        }
        return None;
    }
}

/// A wrapper around a TCP socket (TcpStream) for writing byte stream into
/// Bytes and for parsing Bytes into frames
pub struct Connection {
    socket: TcpStream,
}

impl Connection {
    /// Instantiate a new connection
    pub fn new(socket: TcpStream) -> Self {
        return Self { socket };
    }

    /// Read bytes from the TcpStream, then parse it. If there is a valid
    /// Frame in the bytes read, then return it. Else return None.
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Box<dyn Error>> {
        let mut buf = BytesMut::with_capacity(4096);

        loop {
            // TODO: unnecessary copy but oh well
            if let Some(frame) = Frame::parse(&mut Bytes::from(buf.to_vec())) {
                return Ok(Some(frame));
            }

            self.socket.readable().await?;
            let nbytes = self.socket.read_buf(&mut buf).await?;
            if nbytes == 0 {
                return Ok(None);
            }
        }
    }

    /// Convert the input frame into bytes, then write into the socket
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<usize, Box<dyn Error>> {
        self.socket.writable().await?;
        let nbytes = self.socket.write(&frame.serialize()).await?;
        return Ok(nbytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_serialization() {
        let simple = Frame::Simple("OK".into());
        let expected: Vec<u8> = b"+OK\r\n".to_vec();
        assert_eq!(simple.serialize(), Bytes::copy_from_slice(&expected));
    }

    #[test]
    fn test_integer_serialization() {
        let num = Frame::Integer(0);
        let expected = b":0\r\n";
        assert_eq!(num.serialize(), Bytes::copy_from_slice(expected));
    }

    #[test]
    fn test_bulk_string_serialization() {
        let bulk = Frame::Bulk(Bytes::from(vec![b'6', b'9', b'4', b'2', b'0']));
        let expected = b"$5\r\n69420\r\n";
        assert_eq!(bulk.serialize(), Bytes::copy_from_slice(expected));
    }

    #[test]
    fn test_null_serialization() {
        let null = Frame::Null;
        assert_eq!(null.serialize(), Bytes::from(b"$-1\r\n".to_vec()));
    }

    #[test]
    fn test_array_serialization() {
        let set = Frame::Simple("SET".into());
        let key = Frame::Bulk(Bytes::from("foo"));
        let val = Frame::Bulk(Bytes::from("bar"));
        let cmd = Frame::Array(vec![set, key, val]);
        assert_eq!(
            cmd.serialize(),
            Bytes::from(b"*3\r\n+SET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_vec())
        );
    }

    #[test]
    fn test_simple_string_deserialization() {
        assert_eq!(
            Frame::parse(&mut Bytes::from("+SET\r\n")),
            Some(Frame::Simple("SET".into())),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from("+SET\r\n+++++")),
            Some(Frame::Simple("SET".into())),
        );

        assert_eq!(Frame::parse(&mut Bytes::from("+SET\r")), None,);

        assert_eq!(
            Frame::parse(&mut Bytes::from("+\r\n")),
            Some(Frame::Simple("".into())),
        );
    }

    #[test]
    fn test_error_deserialization() {
        assert_eq!(
            Frame::parse(&mut Bytes::from("-Key not found\r\n")),
            Some(Frame::Error("Key not found".into())),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from("-Key not found\r\n-----")),
            Some(Frame::Error("Key not found".into())),
        );

        assert_eq!(Frame::parse(&mut Bytes::from("-Key not found\r")), None,);

        assert_eq!(
            Frame::parse(&mut Bytes::from("-\r\n")),
            Some(Frame::Error("".into())),
        );
    }

    #[test]
    fn test_integer_deserialization() {
        assert_eq!(
            Frame::parse(&mut Bytes::from(":0\r\n")),
            Some(Frame::Integer(0)),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from(":1\r\n")),
            Some(Frame::Integer(1)),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from(":-1\r\n")),
            Some(Frame::Integer(-1)),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from(":9223372036854775807\r\n")),
            Some(Frame::Integer(9223372036854775807)),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from(":-9223372036854775808\r\n")),
            Some(Frame::Integer(-9223372036854775808)),
        );

        assert_eq!(
            Frame::parse(&mut Bytes::from(":9223372036854775808\r\n")),
            None,
        );
    }

    #[test]
    fn test_bulk_string_deserialization() {
        // Empty bulk string
        assert_eq!(
            Frame::parse(&mut Bytes::from("$0\r\n\r\n")),
            Some(Frame::Bulk(Bytes::new())),
        );

        // Non-empty bulk string
        assert_eq!(
            Frame::parse(&mut Bytes::from("$36\r\n那么古尔丹，代价是什么呢\r\n")),
            Some(Frame::Bulk(Bytes::from("那么古尔丹，代价是什么呢")))
        );

        // Binary unsafe string
        assert_eq!(
            Frame::parse(&mut Bytes::from(b"$2\r\n\r\n\r\n".to_vec())),
            Some(Frame::Bulk(Bytes::from(b"\r\n".to_vec())))
        );

        // Incomplete
        assert_eq!(Frame::parse(&mut Bytes::from("$10\r\n0123456789")), None);

        // Inconsistent number
        assert_eq!(Frame::parse(&mut Bytes::from("$10\r\n0123456\r\n")), None,);

        // Noise at the end
        assert_eq!(
            Frame::parse(&mut Bytes::from("$10\r\n0123456789\r\nxxxxxx")),
            Some(Frame::Bulk(Bytes::from("0123456789")))
        );
    }

    #[test]
    fn test_null_frame_deserialization() {
        assert_eq!(Frame::parse(&mut Bytes::from("$-1\r\n")), Some(Frame::Null));
    }

    #[test]
    fn test_array_deserialization() {
        assert_eq!(
            Frame::parse(&mut Bytes::from("*0\r\n")),
            Some(Frame::Array(vec![]))
        );

        let some_cmd = Frame::Array(vec![
            Frame::Simple("SET".into()),
            Frame::Bulk(Bytes::from("foo")),
            Frame::Bulk(Bytes::from("bar")),
        ]);
        assert_eq!(Frame::parse(&mut some_cmd.serialize()), Some(some_cmd),);

        let some_cmd = Frame::Array(vec![
            Frame::Simple("DEL".into()),
            Frame::Array(vec![
                Frame::Bulk(Bytes::from("key1")),
                Frame::Bulk(Bytes::from("key2")),
                Frame::Bulk(Bytes::from("key3")),
                Frame::Bulk(Bytes::from("key4")),
            ]),
        ]);
        assert_eq!(Frame::parse(&mut some_cmd.serialize()), Some(some_cmd),);

        assert_eq!(Frame::parse(&mut Bytes::from("*3\r\n:0\r\n:1\r\n")), None);

        assert_eq!(
            Frame::parse(&mut Bytes::from("*2\r\n:0\r\n:1\r\n+++++++")),
            Some(Frame::Array(vec![Frame::Integer(0), Frame::Integer(1)])),
        );
    }
}
