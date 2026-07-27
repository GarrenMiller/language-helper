mod server;
mod handlers;
mod morphology;

use server::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
    println!("Hello, world!");
}
