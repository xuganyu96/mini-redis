use clap::{Parser, Subcommand};
use mini_redis::client;

/// A simple redis client
#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    subcmd: SubArgs,

    /// Defaults to "127.0.0.1"
    #[arg()]
    host: Option<String>,

    /// Defaults to 6379
    #[arg()]
    port: Option<usize>,
}

#[derive(Debug, Subcommand)]
enum SubArgs {
    /// Set a key-value pair
    Set(Set),

    /// Get the value of the input key if available
    Get(Get),
}

#[derive(Debug, Parser)]
struct Set {
    key: String,
    val: String,
}

#[derive(Debug, Parser)]
struct Get {
    key: String,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();
    let host = args.host.unwrap_or("127.0.0.1".to_string());
    let port = args.port.unwrap_or(6379);
    let addr = format!("{host}:{port}");

    let mut client = client::connect(&addr).await.unwrap();

    match args.subcmd {
        SubArgs::Set(set) => {
            client.set(&set.key, set.val.into()).await.unwrap();
            println!("Set something");
        },
        SubArgs::Get(get) => {
            if let Some(val) = client.get(&get.key).await.unwrap() {
                println!("{}", String::from_utf8(val.to_vec()).unwrap());
            }
        }
    }
}
