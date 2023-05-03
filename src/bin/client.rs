use redis::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    client.set("foo", "bar").await.unwrap();
    println!("Set 'foo' to 'bar'");
    println!("Get foo: {:?}", client.get("foo").await.unwrap());
    client.set("foo", "baz").await.unwrap();
    println!("Set 'foo' to 'baz'");
    println!("Get foo: {:?}", client.get("foo").await.unwrap());
    println!("{:?} keys deleted", client.del("foo").await.unwrap());
    println!("Get foo: {:?}", client.get("foo").await.unwrap());
}
