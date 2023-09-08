use actix_web::{HttpResponse, web::Query};
use mongodb::bson::{doc, Bson};
use serde_json::{Value, to_value, json, to_string_pretty};
use crate::service::{mongo::LEADERBOARD_COLLECTION, models::*};

pub async fn leaderboard_overview(hash: String) -> HttpResponse {
    let collection = LEADERBOARD_COLLECTION.get().unwrap();
    if let Ok(leaderboard) = collection.find_one(doc! { "hash": &hash }, None).await {
        if !leaderboard.is_none() {
            let mut leaderboard = leaderboard.unwrap();
            leaderboard.remove("_id");

            let _scores = leaderboard.get_document("scores").unwrap();
            // Finish later, "a.json" has new format

            return HttpResponse::Ok()
                .insert_header(("access-control-allow-origin", "*"))
                .body("a");
        }
    }

    HttpResponse::NotFound()
        .insert_header(("access-control-allow-origin", "*"))
        .body("Failed to find leaderboard")
}

pub async fn leaderboard(hash: String, query: Query<ScoresQueryModel>) -> HttpResponse {
    let collection = LEADERBOARD_COLLECTION.get().unwrap();
    if let Ok(leaderboard) = collection.find_one(doc! { "hash": &hash }, None).await {
        if !leaderboard.is_none() {
            let leaderboard = leaderboard.unwrap();
            let score_array = leaderboard.get_document("scores").unwrap();

            if let Ok(characteristic) = score_array.get_document(&query.characteristic) {
                if let Ok(difficulty) = characteristic.get_array(&query.difficulty) {

                    let mut scores: Vec<Value> = difficulty.iter().filter_map(|score| {
                        if let Bson::Document(doc) = score {
                            Some(to_value(doc).unwrap())
                        }
                        else {
                            None
                        }
                    }).collect();
                    scores.sort_by(|a, b| {
                        let acc_a = a["accuracy"].as_f64().unwrap();
                        let acc_b = b["accuracy"].as_f64().unwrap();
                        acc_b.partial_cmp(&acc_a).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    for score in &mut scores {
                        let score = score.as_object_mut().unwrap();
                        let id = score.get("id").unwrap().to_string();
                        score.insert("username".to_string(), Value::String(crate::utils::user::get::get_username(id).await));
                    
                        if query.sort == "around" {
                            // do
                        }
                    }

                    let mut limit: usize = query.limit.try_into().unwrap();
                    if limit > 50 {
                        limit = 50;
                    }
                    let page: usize = query.page.try_into().unwrap();
                    let start = page * limit;
                    let end = std::cmp::min(start + limit, scores.len());

                    let limited_scores: Vec<Value> = scores[start..end].to_vec();
                    let score_count = &scores.len();
                    let response = json!({
                        "scoreCount": score_count,
                        "scores": &limited_scores
                    });
                    return HttpResponse::Ok()
                        .insert_header(("access-control-allow-origin", "*"))
                        .body(to_string_pretty(&response).unwrap());
                }
            }
        }
    }

    HttpResponse::NotFound()
        .insert_header(("access-control-allow-origin", "*"))
        .body("Failed to find leaderboard")
}