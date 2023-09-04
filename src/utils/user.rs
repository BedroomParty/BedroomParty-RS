use actix_web::{HttpResponse, web};
use rand::Rng;
use serde_json::to_string_pretty;
use mongodb::bson::doc;
use crate::service::mongo::*;
use base64::{Engine as _, engine::general_purpose};

use super::models::*;

pub async fn get_user_info(id: u32) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            let mut user = user.unwrap();
            user.remove("apiKey");
            user.remove("sessionKey");
            HttpResponse::Ok().body(to_string_pretty(&user).unwrap().to_string());
        }
    }
    HttpResponse::NotFound().body(format!("User {} not found!", id))
}

pub async fn get_api_key(id: i64) -> String {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            let session_key = user.unwrap().get("sessionKey").unwrap().to_string().replace("\"", "");
            return session_key;
        }
    }
    "No user found".to_string()
    
}
pub async fn get_api_key_time(id: i64) -> f64 {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            let user = user.unwrap();
            return user.get("sessionKeyExpires").unwrap().as_f64().unwrap();
        }
    }
    0.0
}

pub async fn login_user(body: web::Json<UserLoginModel>) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": body.user_id }, None).await {
        if !user.is_none() {
            let user = user.unwrap();
            let mut api_key = user.get("apiKey").unwrap().to_string();
            api_key = api_key.replace("\"", "");
            if api_key == body.api_key {
                let session_key = general_purpose::STANDARD.encode(generate_random_string(50));
                user.get("username").unwrap();
                let time_set = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
                let update = doc! {
                    "$set": { 
                        "sessionKey": &session_key,
                        "sessionKeyExpires": time_set + 21600.0
                    }
                };
                collection.update_one(doc! { "game_id": &body.user_id }, update, None).await.unwrap();

                println!("[API: /user/login] {} has logged in, session key: {}", user.get("username").unwrap().to_string().replace("\"", ""), &session_key);
                return HttpResponse::Ok().body(doc! { "sessionKey": &session_key }.to_string());
            }
        }
    }
    HttpResponse::Unauthorized().body("Either user doesn't exist or API key doesn't match")
}

pub async fn create_new_user(body: web::Json<UserModel>) -> HttpResponse {    
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": &body.game_id }, None).await {
        if user.is_none() {
            
            let api_key = general_purpose::STANDARD.encode(generate_random_string(50));

            let new_user = doc! {
                "discord_id": &body.discord_id,
                "game_id": &body.game_id,
                "username": &body.username,
                "avatar": &body.avatar,
                "apiKey": api_key,
                "sessionKey": ""
            };
            collection.insert_one(new_user, None).await.unwrap();
            println!("[API: /user/create] Created new user with username: {}", &body.username);
            return HttpResponse::Ok().body("Created new user!");
        }
    }
    HttpResponse::Conflict().body("User already exists!")
}

fn generate_random_string(length: i32) -> String {
    let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let index = rng.gen_range(0..characters.len());
            characters.chars().nth(index).unwrap()
        }).collect();
    random_string
}