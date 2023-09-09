use actix_web::{HttpResponse, web::Json};
use mongodb::bson::doc;
use serde_json::Value;
use std::time::SystemTime;
use crate::service::{mongo::LEADERBOARD_COLLECTION, models::ScoreModel};

pub async fn score_upload(hash: String, body: Json<ScoreModel>) -> HttpResponse {
    let collection = LEADERBOARD_COLLECTION.get().unwrap();
    if let Ok(leaderboard) = collection.find_one(doc! { "hash": &hash}, None).await {
        let time_set: i64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().try_into().unwrap();
        let new_score = doc! {
            "id": &body.id.to_string(),
            "modifiedScore": &body.modifiedScore,
            "multipliedScore": &body.multipliedScore,
            "accuracy": &body.accuracy,
            "misses": &body.misses,
            "badCuts": &body.badCuts,
            "fullCombo": &body.fullCombo,
            "modifiers": &body.modifiers,
            "timeSet": time_set
        };

        if leaderboard.is_none() {
            println!("[API: /leaderboard/{}/upload] Creating new leaderboard", &hash);

            let request: Value = reqwest::get(format!("https://api.beatsaver.com/maps/hash/{}", &hash)).await.unwrap().json().await.unwrap();
            if !request["error"].is_null() {
                return HttpResponse::Forbidden().body("Leaderboard doesn't exist on BeatSaver, upload is forbidden");
            }

            let new_leaderboard = doc! {
                "name": request["name"].as_str().unwrap(),
                "hash": &hash,
                "scores": {
                    &body.characteristic: {
                        &body.difficulty.to_string(): [
                            &new_score
                        ]
                    }
                }
            };

            collection.insert_one(new_leaderboard, None).await.unwrap();
            return HttpResponse::Ok().body("Created new leaderboard and uploaded score.");
        }
        else {
            println!("[API: /leaderboard/{}/upload] Attempting upload...", &hash);

            let leaderboard = leaderboard.unwrap();
            let directory = format!("scores.{}.{}", &body.characteristic, &body.difficulty);
            let scores = leaderboard.get_document("scores").unwrap();
            if let Ok(characteristic) = scores.get_document(&body.characteristic) {
                if let Ok(difficulty) = characteristic.get_array(&body.difficulty.to_string()) {
                    for score in difficulty {
                        let score = score.as_document().unwrap();

                        if score.get_str("id").unwrap() == body.id {
                            if score.get_f64("accuracy").unwrap() < body.accuracy.try_into().unwrap() {
                                let score_pull = doc! {
                                    "$pull": {
                                        &directory: {
                                            "id": &body.id
                                        }
                                    }
                                };
                                
                                collection.update_one(doc! { "hash": &hash }, score_pull, None).await.unwrap(); // Deletes current score
                            }
                            else  {
                                return HttpResponse::Conflict().body("Not a highscore");
                            }
                        }
                    }
                }
                let score_push = doc! {
                    "$push": {
                        &directory: &new_score
                    }
                };
                collection.update_one(doc! { "hash": &hash }, score_push, None).await.unwrap();
                println!("[API: /leaderboard/{}/upload] Successfully uploaded", &hash);
                return HttpResponse::Ok().body("Score uploaded");
            }
        }
    }
    HttpResponse::InternalServerError().body("Something failed me no kno y")
}