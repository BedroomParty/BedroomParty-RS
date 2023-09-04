mod utils;
mod service;

#[tokio::main]
async fn main() {
    println!("Running Services");
    dotenv::dotenv().ok();
    service::mongo::setup().await.expect("Failed to setup mongo!");
    service::server::setup().await.expect("Failed to setup server!");
}
