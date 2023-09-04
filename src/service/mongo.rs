use once_cell::sync::OnceCell;
use mongodb::{Client, Collection, bson::Document, options::ClientOptions};

pub static LEADERBOARD_COLLECTION: OnceCell<Collection<Document>> = OnceCell::new();
pub static USER_COLLECTION: OnceCell<Collection<Document>> = OnceCell::new();

pub async fn setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to MongoDB...");

    let mongo_uri = std::env::var("MONGO_URI").expect("Failed MONGO_URI");
    let client_options = ClientOptions::parse(mongo_uri).await.expect("Failed Parsing");
    let client = Client::with_options(client_options).expect("Failed Creating");
    let db = std::env::var("DATABASE").expect("Failed DATABASE");

    LEADERBOARD_COLLECTION.set(client.database(&db).collection("leaderboards")).expect("Failed LEADERBOARD_COLLECTION");
    USER_COLLECTION.set(client.database(&db).collection("users")).expect("Failed USER_COLLECTION");

    println!("Collections set up successfully.");

    Ok(())
}
