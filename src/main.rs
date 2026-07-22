mod server;
mod handlers;

use server::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
    println!("Hello, world!");
}
