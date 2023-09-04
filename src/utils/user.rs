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
            HttpResponse::Ok().body(to_string_pretty(&user).unwrap().to_string());
        }
    }
    HttpResponse::NotFound().body(format!("User {} not found!", id))
}

pub async fn login_user(body: web::Json<UserLoginModel>) -> HttpResponse {
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": body.user_id }, None).await {
        if !user.is_none() {
            let user = user.unwrap();
            let mut api_key = user.get("apiKey").unwrap().to_string();
            api_key = api_key.replace("\"", "");
            if api_key == body.api_key {
                println!("[API: /user/login] {} has logged in", user.get("username").unwrap().to_string().replace("\"", ""));
                return HttpResponse::Ok().body("User has successfully logged in!");
            }
        }
    }
    HttpResponse::Unauthorized().body("Either user doesn't exist or API key doesn't match")
}

pub async fn create_new_user(body: web::Json<UserModel>) -> HttpResponse {    
    let collection = USER_COLLECTION.get().unwrap();
    if let Ok(user) = collection.find_one(doc! { "game_id": &body.game_id }, None).await {
        if user.is_none() {
            let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let mut rng = rand::thread_rng();
            let random_string: String = (0..50)
                .map(|_| {
                    let index = rng.gen_range(0..characters.len());
                    characters.chars().nth(index).unwrap()
                }).collect();
            let api_key = general_purpose::STANDARD.encode(random_string);

            let new_user = doc! {
                "discord_id": &body.discord_id,
                "game_id": &body.game_id,
                "username": &body.username,
                "avatar": &body.avatar,
                "apiKey": api_key
            };
            collection.insert_one(new_user, None).await.unwrap();
            println!("[API: /user/create] Created new user with username: {}", &body.username);
            return HttpResponse::Ok().body("Created new user!");
        }
    }
    HttpResponse::Conflict().body("User already exists!")
}