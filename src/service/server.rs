use std::{fs::File, io::Read, time::{SystemTime, UNIX_EPOCH}};
use actix_web::{HttpServer, App, Responder, HttpResponse, get, post, web::{Path, self}, HttpRequest};
use crate::{
    service::models::*,
    utils::{
        leaderboard::{
            get::*,
            post::*
        },
        user::{
            get::*,
            post::*
        }
    }
};


pub async fn setup() -> std::io::Result<()> {
    println!("Starting Server...");

    let client = HttpServer::new(|| {
        App::new()
            .service(get_status)
            .service(get_staff_ids)
            .service(get_fucking_docs)
            .service(get_fucking_swagger)
            //.service(get_fucking_content) keep just in case swagger acts up again

            .service(post_user_create)
            .service(post_user_login)
            .service(post_user_api_key)
            .service(get_user_profile)
            .service(get_user_avatar)

            .service(post_score_upload)
            .service(get_leaderboard)
            .service(get_leaderboard_overview)
    });
    println!("Server set up successfully.");
    client.bind(("127.0.0.1", 8080))?
        .workers(8)
        .run()
        .await
}

#[get("/")]
async fn get_status() -> impl Responder {
    HttpResponse::Ok().body("Online baby")
}

#[get("/docs")]
async fn get_fucking_docs() -> impl Responder {
    let content = std::fs::read_to_string("./src/extras/dist/index.html").unwrap();
    
    HttpResponse::Ok().content_type("text/html").body(content)
}

#[get("/{file}")]
async fn get_fucking_content(file: Path<String>) -> impl Responder {
    let content = std::fs::read_to_string(format!("./src/extras/dist/{}", file.to_string())).expect("failed lmfao");
    
    if content.is_empty() {
        HttpResponse::Ok().body("a")
    }
    else {
        HttpResponse::Ok().body(content)
    }
}

#[get("/docs/swagger.json")]
async fn get_fucking_swagger() -> impl Responder {
    let content = std::fs::read_to_string("./src/extras/swagger.json").unwrap();
    HttpResponse::Ok().body(content)
}

#[get("/staff")]
async fn get_staff_ids() -> impl Responder {
    HttpResponse::Ok().body(std::fs::read_to_string("./src/extras/StaffIDs.txt").unwrap())
}

#[get("/user/{id}")]
async fn get_user_profile(id: Path<String>) -> impl Responder {
    get_user(id.to_string()).await
}

#[get("/user/{id}/avatar")]
async fn get_user_avatar(id: Path<String>) -> impl Responder {
    if let Ok(mut avatar) = File::open(format!("./src/extras/Users/Avatars/{id}.png")) {
        let mut avatar_data = Vec::new();
        avatar.read_to_end(&mut avatar_data).unwrap();
        return HttpResponse::Ok().content_type("image/png").body(avatar_data);
    }

    HttpResponse::NotFound().body("Unable to find avatar")
}

#[post("/user/create")]
async fn post_user_create(request: HttpRequest, body: web::Json<UserModel>) -> impl Responder {
    if let Some(authentication) = request.headers().get("Authorization") {
        if authentication.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            return create_user(body).await
        }
    }
    HttpResponse::Unauthorized().body("Authorization is either null or doesn't match!")
}

#[post("/user/login")]
async fn post_user_login(body: web::Json<UserLoginModel>, request: HttpRequest) -> impl Responder {
    if let Some(authorization) = request.headers().get("Authorization") {
        if authorization.to_str().unwrap() == get_api_key(body.user_id.to_string()).await.as_str() {
            return login_user(body).await;
        }
    }
    HttpResponse::Unauthorized().body("Authorization may not match, or be null.")
}

#[post("/user/{id}/apikey")]
async fn post_user_api_key(request: HttpRequest, id: Path<String>) -> impl Responder {
    if let Some(authorization) = request.headers().get("Authorization") {
        if authorization.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            return HttpResponse::Ok().body(mongodb::bson::doc! { "apiKey": get_api_key(id.to_string()).await }.to_string());
        }
    }
    HttpResponse::Unauthorized().body("Authorzation is either null or doesn't match!")
}

#[post("/user/{id}/avatar/upload")]
async fn post_upload_avatar(id: Path<String>, request: HttpRequest, body: web::Json<AvatarUploadModel>) -> impl Responder {
    if let Some(authorization) = request.headers().get("Authorization") {
        if authorization.to_str().unwrap() == std::env::var("PRIVATE_AUTH").unwrap() {
            upload_avatar(body.avatar.to_string(), id.to_string()).await
        }
    }

    HttpResponse::Ok()
}

// Leaderboards
#[post("/leaderboard/{hash}/upload")]
async fn post_score_upload(request: HttpRequest, body: web::Json<ScoreModel>, hash: Path<String>) -> impl Responder {
    if let Some(authentication) = request.headers().get("Authorization") {
        if authentication.to_str().unwrap() == get_session_key(body.id.to_string()).await.as_str() {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() < get_session_key_time(body.id.to_string()).await.try_into().unwrap() {
                return score_upload(hash.to_string(), body).await;
            }
        }
    }
    HttpResponse::Unauthorized().body("Authoraization is either null or doesn't match!")
}

#[get("/leaderboard/{hash}")]
async fn get_leaderboard(hash: Path<String>, query: web::Query<ScoresQueryModel>) -> impl Responder {
    leaderboard(hash.to_string(), query).await
}

#[get("/leaderboard/{hash}/overview")]
async fn get_leaderboard_overview(hash: Path<String>) -> impl Responder {
    leaderboard_overview(hash.to_string()).await
} 
