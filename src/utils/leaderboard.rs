use actix_web::{HttpResponse, web};
use mongodb::bson::{doc, Bson};
use serde_json::{json, Value};

use crate::service::mongo::*;

use super::models::{ScoreModel, ScoresQueryModel};

pub async fn get_leaderboard(hash: String) -> HttpResponse {
    if let Some(collection) = LEADERBOARD_COLLECTION.get() {
        if let Ok(map) = collection.find_one(doc!{ "hash": &hash }, None).await {
            if !map.is_none() {
                let mut map = map.unwrap();
                map.remove("_id");
                let pretty = serde_json::to_string_pretty(&map).unwrap();
                return HttpResponse::Ok().insert_header(("access-control-allow-origin", "*")).body(pretty);
            }
        }
    }

    HttpResponse::NotFound().body(format!("Failed to find {}", hash))
}

pub async fn get_scores(hash: String, query: web::Query<ScoresQueryModel>) -> HttpResponse {
    let collection = LEADERBOARD_COLLECTION.get().unwrap();
    if let Ok(leaderboard) = collection.find_one(doc! { "hash": hash }, None).await {
        if !leaderboard.is_none() {
            let leaderboard = leaderboard.unwrap();
            let scores = leaderboard.get_document("scores").unwrap();
            if let Ok(characteristic) = scores.get_document(&query.characteristic) {
                if let Ok(difficulty) = characteristic.get_array(&query.difficulty.to_string()) {
                    let mut sorted_scores: Vec<Value> = difficulty.to_vec().iter().filter_map(|score| {
                        if let Bson::Document(doc) = score {
                            Some(serde_json::to_value(doc).unwrap())
                        }
                        else {
                            None
                        }
                    }).collect();
                    sorted_scores.sort_by(|a, b| {
                        let acc_a = a["accuracy"].as_f64().unwrap();
                        let acc_b = b["accuracy"].as_f64().unwrap();
                        acc_b.partial_cmp(&acc_a).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let mut  query_limit: usize = query.limit.try_into().unwrap();
                    if query_limit > 50 {
                        query_limit = 50;
                    }
                    let query_page: usize = query.page.try_into().unwrap();
                    let start_index = query_page * query_limit;
                    let end_index = std::cmp::min(start_index + query_limit, sorted_scores.len());

                    let limit: Vec<Value> = sorted_scores[start_index..end_index].to_vec();

                    let score_count = &sorted_scores.len();
                    let response = json!({
                        "scoreCount": &score_count,
                        "scores": &limit
                    });
                    return HttpResponse::Ok().insert_header(("access-control-allow-origin", "*")).body(serde_json::to_string_pretty(&response).unwrap())
                }
            }
        }
    }

    HttpResponse::NotFound().body("Leaderboard not found")
}

pub async fn upload_score(body: web::Json<ScoreModel>) -> HttpResponse {
    let collection = LEADERBOARD_COLLECTION.get().unwrap();
    let leaderboard = collection.find_one(doc! { "hash": &body.hash }, None).await.unwrap();

    let score = doc! {
        "id": &body.id,
        "modifiedScore": &body.modifiedScore,
        "multipliedScore": &body.multipliedScore,
        "accuracy": &body.accuracy,
        "misses": &body.misses,
        "badCuts": &body.badCuts,
        "fullCombo": &body.fullCombo,
        "modifiers": &body.modifiers,
    };

    if leaderboard.is_none() {
        println!("[API: /leaderboard/{}/upload] Creating New Leaderboard", &body.hash);
        let characteristic = &body.characteristic;
        let difficulty = &body.difficulty.to_string();
        let new_leaderboard = doc! {
            "hash": &body.hash,
            "scores": {
                characteristic: {
                    difficulty: [
                        score
                    ]
                }
            }
        };
        collection.insert_one(new_leaderboard, None).await.unwrap();

        return HttpResponse::Ok().body("Created leaderboard and uploaded score successfully");
    }
    else {
        println!("[API: /leaderboard/{}/upload] Uploading Score to Existing Leaderboard", &body.hash);
        let directory = format!("scores.{}.{}", &body.characteristic, &body.difficulty);
        let leaderboard = leaderboard.unwrap();
        let scores = leaderboard.get_document("scores").unwrap();
        if let Ok(characteristic) = scores.get_document(&body.characteristic) {
            if let Ok(difficulty) = characteristic.get_array(&body.difficulty.to_string()) {

                for score in difficulty {
                    let score = score.as_document().unwrap();
                    if score.get_i64("id").unwrap() == body.id  {
                        if score.get_i64("modifiedScore").unwrap() < body.modifiedScore || score.get_i64("multipliedScore").unwrap() < body.multipliedScore {
                            let update = doc! {
                                "$pull": {
                                    &directory: {
                                        "id": body.id,
                                    }
                                }
                            };

                            collection.update_one(doc! { "hash": &body.hash }, update, None).await.unwrap();
                        }
                        else { return HttpResponse::Conflict().body("Not a new highscore"); }
                    }
                }
            }
        }

        let update = doc! {
            "$push": {
                &directory: score
            }
        };

        collection.update_one(doc! { "hash": &body.hash}, update, None).await.unwrap();
    }

    HttpResponse::Ok().body("Successfully uploaded score")
}