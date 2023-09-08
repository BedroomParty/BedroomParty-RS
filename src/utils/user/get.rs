use actix_web::HttpResponse;
use serde_json::to_string_pretty;
use mongodb::bson::doc;
use crate::service::mongo::USER_COLLECTION;

pub async fn get_user(id: String) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            let mut user = user.unwrap();
            user.remove("_id");
            user.remove("apiKey");
            user.remove("sessionKey");
            user.remove("sessionKeyExpires");
            return HttpResponse::Ok()
                .insert_header(("access-control-allow-origin", "*"))
                .body(to_string_pretty(&user).unwrap().to_string());
        }
    }

    HttpResponse::NotFound()
        .insert_header(("access-control-allow-origin", "*"))
        .body("User not found")
}

pub async fn get_username(id: String) -> String {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id.replace("\"", "") }, None).await {
        if !user.is_none() {
            return user.unwrap().get_str("username").unwrap().to_string();
        }
    }
    "null".to_string()
}

pub async fn get_session_key(id: String) -> String {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            return user.unwrap().get_str("sessionKey").unwrap().to_string();
        }
    }
    "null".to_string()
}

pub async fn get_api_key(id: String) -> String {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            return user.unwrap().get_str("apiKey").unwrap().to_string();
        }
    }
    "null".to_string()
}


pub async fn get_session_key_time(id: String) -> i64 {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            return user.unwrap().get_i64("sessionKeyExpires").unwrap();
        }
    }
    0
}