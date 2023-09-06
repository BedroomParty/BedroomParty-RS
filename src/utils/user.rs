use std::fs::File;
use std::io::{self, Write, Cursor};
use image::io::Reader as ImageReader;
use std::io::copy;
use actix_web::{HttpResponse, web, HttpRequest};
use rand::Rng;
use serde_json::{to_string_pretty, Value};
use mongodb::bson::doc;
use crate::service::mongo::*;
use base64::{Engine as _, engine::general_purpose};

use super::models::*;

pub async fn get_user_info(id: i64) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            let mut user = user.unwrap();
            user.remove("_id");
            user.remove("apiKey");
            user.remove("sessionKey");
            user.remove("sessionKeyExpires");
            return HttpResponse::Ok().body(to_string_pretty(&user).unwrap().to_string());
        }
    }
    HttpResponse::NotFound().body(format!("User {} not found!", id))
}

pub async fn get_user_name(id: i64) -> String {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": id }, None).await {
        if !user.is_none() {
            return user.unwrap().get_str("username").unwrap().to_string();
        }
    }
    "null".to_string()
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

pub async fn login_user(body: web::Json<UserLoginModel>, request: HttpRequest) -> HttpResponse {
    if request.headers().get("Authorization").is_none() {
        return HttpResponse::Unauthorized().await.unwrap();
    }

    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": body.user_id }, None).await {
        if !user.is_none() {
            let user = user.unwrap();
            let mut api_key = user.get("apiKey").unwrap().to_string();
            api_key = api_key.replace("\"", "");
            if api_key.as_str() == request.headers().get("Authorization").unwrap().to_str().unwrap() {
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

            let request: Value = reqwest::get(format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key=E84E7BFB414CCF2EEAAF9FDCC337CAF7&steamids={}", &body.game_id)).await.unwrap().json().await.unwrap();
            let url = request["response"]["players"][0]["avatarfull"].as_str().unwrap();

            let avatar_res = reqwest::get(url).await.unwrap();
            let mut file = File::create(format!("./src/extras/Users/Avatars/{}.png", body.game_id)).unwrap();
            let mut content = Cursor::new(avatar_res.bytes().await.unwrap());
            copy(&mut content, &mut file).unwrap();

            let new_user = doc! {
                "discord_id": &body.discord_id,
                "game_id": &body.game_id,
                "username": &body.username,
                "avatar": format!("https://api.thebedroom.party/user/{}/avatar", &body.game_id),
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