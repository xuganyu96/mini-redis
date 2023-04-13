//! Underlying components of mini Redis
use tokio::io::{ self, AsyncReadExt };
use tokio::net::TcpStream;
use mini_redis::{ self, Frame };

struct Connection {
    socket: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    /// Initialize the redis connection including an empty buffer and the 
    /// cursor. Page size is hard-coded and hidden from the caller
    fn new(socket: TcpStream) -> Self {
        return Self {
            socket,
            buffer: vec![0; 4096],
            cursor: 0,
        };
    }

    /// Try to read a single frame from the underlying TCP stream. Return
    /// the frame if there is a complete, otherwise return an empty frame or
    /// report error
    async fn read_frame(&mut self) -> mini_redis::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame() {
                return Ok(Some(frame));
            }

            // try to read more data, but before we can read more data, need to
            // check whether we have buffer space left:
            if self.cursor == self.buffer.len() {
                self.buffer.resize(self.cursor * 2, 0);
            }

            match self.socket.read(&mut self.buffer[self.cursor..]).await? {
                0 if self.cursor == 0 => {
                    return Ok(None);  // no frame is read
                },
                0 if self.cursor > 0 => {
                    eprintln!("Connection reset unexpectedly");
                    return Err("Connection reset unexpectedly".into());
                },
                n => {
                    self.cursor += n;
                }
            }
        }
    }

    async fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        todo!();
    }

    /// Check if the internal buffer constitutes a complete frame. If yes,
    /// return the frame and clear the buffer. If no, return None
    fn parse_frame(&self) -> Option<Frame> {
        todo!();
    }
}
