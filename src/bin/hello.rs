use tokio::net::{ TcpListener, TcpStream };
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    let (mut socket, _addr) = listener.accept().await.unwrap();

    if socket.writable().await.is_ok() {
        let buffer = "那么古尔丹，代价是什么呢\n".to_string();
        socket.write_all(buffer.as_bytes()).await.unwrap();
    }

    if socket.readable().await.is_ok() {
        let mut buffer: Vec<u8> = vec![0;24];
        socket.read(&mut buffer).await.unwrap();

        let msg = String::from_utf8(buffer).unwrap();
        println!("{msg}");
    }
}
