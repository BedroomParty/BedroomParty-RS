use std::time::SystemTime;
use mongodb::bson::doc;
use actix_web::{web::Json, HttpResponse};
use base64::{Engine as _, engine::general_purpose};
use image::EncodableLayout;
use serde_json::Value;
use crate::service::models::*;

use crate::service::mongo::USER_COLLECTION;

pub async fn upload_avatar(avatar: String, id: String) {
    let avatar = general_purpose::STANDARD.decode(avatar).unwrap();
    let image = image::load_from_memory(avatar.as_bytes()).unwrap();
    image.save_with_format(format!("./src/extras/Users/Avatars/{id}.png"), image::ImageFormat::Png).unwrap();
}

pub async fn login_user(body: Json<UserLoginModel>) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": &body.user_id }, None).await {
        if !user.is_none() {
            let user = user.unwrap();
            let username = user.get_str("username").unwrap();
            let session_key = general_purpose::STANDARD.encode(generate_random_string(50));
            let time_set: i64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().try_into().unwrap();
            let update = doc! {
                "$set": {
                    "sessionKey": &session_key,
                    "sessionKeyExpires": time_set + 21600
                }
            };
            
            collection.update_one(doc! { "game_id": &body.user_id }, update, None).await.unwrap();
            println!("[API: /user/login] {} logged in. Session key: {}", &username, &session_key);
            return HttpResponse::Ok().body(doc! { "sessionKey": &session_key, "username": &username }.to_string());
        }
    }
    HttpResponse::InternalServerError().body("Dunno what happened lmfao, better luck next time bucko")
} 

pub async fn create_user(body: Json<UserModel>) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": &body.game_id}, None).await {
        if user.is_none() {
            let api_key = general_purpose::STANDARD.encode(generate_random_string(50));

            let request: Value = reqwest::get(format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key=E84E7BFB414CCF2EEAAF9FDCC337CAF7&steamids={}", &body.game_id)).await.unwrap().json().await.unwrap();
            let url = request["response"]["players"][0]["avatarfull"].as_str().unwrap();
            let avatar = reqwest::get(url).await.unwrap();

            let image = image::load_from_memory(&avatar.bytes().await.unwrap().to_vec()).unwrap();
            image.save_with_format(format!("./src/extras/Users/Avatars/{}.png", &body.game_id), image::ImageFormat::Png).unwrap();
            
            let expires: i64 = 0;
            let new_user = doc! {
                "discord_id": &body.discord_id,
                "game_id": &body.game_id,
                "username": &body.username,
                "avatar": format!("https://api.thebedroom.party/user/{}/avatar", &body.game_id),
                "apiKey": &api_key,
                "sessionKey": "",
                "sessionKeyExpires": expires
            };
            collection.insert_one(new_user, None).await.unwrap();
            println!("[API: /user/create] Created new user");
            return HttpResponse::Ok().body(doc! { "apiKey": &api_key }.to_string());
        }
    }
    HttpResponse::Conflict().body("User already exists")
}

fn generate_random_string(length: i32) -> String {
    let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let index = rand::Rng::gen_range(&mut rng, 0..characters.len());
            characters.chars().nth(index).unwrap()
        }).collect();
    random_string
}