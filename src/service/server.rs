use std::{fs::File, io::Read, time::{SystemTime, UNIX_EPOCH}};
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
            .service(get_fucking_docs)
            .service(upload_avatar)
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

#[get("/docs")]
async fn get_fucking_docs() -> impl Responder {
    let content = std::fs::read_to_string("./src/extras/swagger.html").unwrap();
    HttpResponse::Ok().content_type("text/html").body(content)
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
    if let Ok(mut avatar) = File::open(format!("./src/extras/Users/Avatars/{id}.png")) {
        let mut avatar_data = Vec::new();
        avatar.read_to_end(&mut avatar_data).unwrap();
        return HttpResponse::Ok().content_type("image/png").body(avatar_data);
    }

    HttpResponse::NotFound().body("Unable to find avatar")
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
async fn user_login(body: web::Json<UserLoginModel>, request: HttpRequest) -> impl Responder {
    user::login_user(body, request).await
}

#[post("/user/{id}/apikey")]
async fn get_user_api_key(request: HttpRequest, id: Path<u64>) -> impl Responder {
    if let Some(authorization) = request.headers().get("Authorization") {
        if authorization.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            return HttpResponse::Ok().body(mongodb::bson::doc! { "apiKey": user::get_api_key((*id).try_into().unwrap()).await}.to_string());
        }
    }
    HttpResponse::Unauthorized().body("Authorzation is either null or doesn't match!")
}

#[post("/user/{id}/avatar/upload")]
async fn upload_avatar(id: Path<u64>, request: HttpRequest, body: web::Json<AvatarUploadModel>) -> impl Responder {
    if let Some(authorization) = request.headers().get("Authorization") {
        if authorization.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            user::upload_avatar(body.avatar.to_string(), (*id).try_into().unwrap()).await;
        }
    }

    HttpResponse::Ok()
}

// Leaderboards
#[post("/leaderboard/{hash}/upload")]
async fn upload_score(request: HttpRequest, body: web::Json<ScoreModel>) -> impl Responder {
    if let Some(authentication) = request.headers().get("Authorization") {
        if authentication.to_str().unwrap() == user::get_api_key(body.id).await.as_str() {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() < user::get_api_key_time(body.id).await {
                return utils::leaderboard::upload_score(body).await;
            }
        }
    }
    HttpResponse::Unauthorized().body("Authoraization is either null or doesn't match!")
}

#[get("/leaderboard/{hash}")]
async fn get_scores(hash: Path<String>, query: web::Query<ScoresQueryModel>) -> impl Responder {
    utils::leaderboard::get_scores(hash.to_string(), query).await
}

#[get("/leaderboard/{hash}/overview")]
async fn get_leaderboard(hash: Path<String>) -> impl Responder {
    utils::leaderboard::get_leaderboard(hash.to_string()).await
} 
