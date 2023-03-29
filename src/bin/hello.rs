async fn say_world() {
    println!("world!");
}

#[tokio::main]
async fn main() {
    let handle = say_world();
    print!("Hello, ");
    handle.await;
}
