//! tcpecho: listens in on a Socket and returns any data that clients send
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;
use clap::Parser;

/// Listens in on a socket and return any data that the clients send
#[derive(Parser, Debug)]
struct Args {
    /// Set the hostname on which to run the server, defaults to 0.0.0.0
    #[arg(long)]
    host: Option<String>,

    /// The port on which to run the server, defaults to 8000
    #[arg(short, long)]
    port: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let addr = format!("{}:{}", args.host.unwrap_or("0.0.0.0".into()), args.port.unwrap_or(8000));
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening at {addr}");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Connected to {addr:?}");
        tokio::spawn(async move {
            let _ = echo(socket, addr).await;
        });
    }
}

async fn echo(mut socket: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut buf = BytesMut::new();

    loop {
        socket.readable().await?;
        let nbytes = socket.read_buf(&mut buf).await?;
        if nbytes == 0 {
            println!("{addr:?} disconnected");
            return Ok(());
        }
        socket.writable().await?;
        socket.write_buf(&mut buf).await?;
    }
}
