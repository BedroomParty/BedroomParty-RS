use std::{fs::File, io::Read};

use actix_web::{HttpServer, App, Responder, HttpResponse, get, post, web::{Path, self}, HttpRequest};
use crate::utils::{models::*, self, user};



pub async fn setup() -> std::io::Result<()> {
    println!("Starting Server...");
    HttpServer::new(|| {
        App::new()
            .service(status)
            .service(get_user_profile)
            .service(get_staff_ids)
            .service(get_scores)
            .service(get_leaderboard)
            .service(get_user_avatar)
            .service(upload_score)
            .service(create_user)
            .service(user_login)
    })
    .bind(("127.0.0.1", 8080))?
    .workers(4)
    .run()
    .await
}

#[get("/")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("Online baby")
}

#[get("/staff")]
async fn get_staff_ids() -> impl Responder {
    HttpResponse::Ok().body(std::fs::read_to_string("./src/extras/StaffIDs.txt").unwrap())
}

#[get("/user/{id}")]
async fn get_user_profile(id: Path<u64>) -> impl Responder {
    utils::user::get_user_info((*id).try_into().unwrap()).await
}

#[get("/user/{id}/avatar")]
async fn get_user_avatar(id: Path<u64>) -> impl Responder {
    let avatar = File::open(format!("./src/extras/Users/Avatars/{id}.png"));
    let mut avatar_data = Vec::new();
    avatar.unwrap().read_to_end(&mut avatar_data).unwrap();

    HttpResponse::Ok().content_type("image/png").body(avatar_data)
}

#[post("/user/create")]
async fn create_user(request: HttpRequest, body: web::Json<UserModel>) -> impl Responder {
    if let Some(authentication) = request.headers().get("Authorization") {
        if authentication.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            return user::create_new_user(body).await;
        }
    }
    HttpResponse::Unauthorized().body("Authorization is either null or doesn't match!")
}

#[post("/user/login")]
async fn user_login(body: web::Json<UserLoginModel>) -> impl Responder {
    user::login_user(body).await
}

// Leaderboards
#[post("/leaderboard/{hash}/upload")]
async fn upload_score(_request: HttpRequest, body: web::Json<ScoreModel>) -> impl Responder {
    //if let Some(authorization) = request.headers().get("Authorization") {
    //    if authorization.is_empty() || authorization.to_str().unwrap() != utils::user::get_api_key(authorization.to_str().unwrap()).await {
            //return HttpResponse::Unauthorized().body("Authorization is either null or doesn't exist");
    //    }
    //}
    utils::leaderboard::upload_score(body).await
}

#[get("/leaderboard/{hash}")]
async fn get_scores(hash: Path<String>, query: web::Query<ScoresQueryModel>) -> impl Responder {
    utils::leaderboard::get_scores(hash.to_string(), query).await
}

#[get("/leaderboard/{hash}/overview")]
async fn get_leaderboard(hash: Path<String>) -> impl Responder {
    utils::leaderboard::get_leaderboard(hash.to_string()).await
} 