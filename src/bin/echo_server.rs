//! Echo server: listens in on a TCP port and writes back any data it receives
use tokio::io::{ self, AsyncReadExt, AsyncWriteExt };
use tokio::net::{ TcpListener, TcpStream };

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    while let Ok((socket, _addr)) = listener.accept().await {
        tokio::spawn(async move {
            echo_manually(socket).await.unwrap();
        });
    }
    return Ok(());
}

#[allow(dead_code)]
async fn echo_with_copy(mut socket: TcpStream) {
    // let (mut reader, mut writer) = io::split(socket);
    let (mut reader, mut writer) = socket.split();
    // let (mut reader, mut writer) = socket.into_split();
    io::copy(&mut reader, &mut writer).await.unwrap();
}

#[allow(dead_code)]
async fn echo_manually(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = vec![0; 8];

    loop {
        match socket.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                return Ok(());
            },
            Ok(n) => {
                socket.write_all(&buffer[..n]).await?;
            },
            Err(e) => {
                eprintln!("echo_server: {e}");
                return Ok(());
            }
        }
    }
}
