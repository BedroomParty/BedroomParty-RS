#![allow(dead_code)]
#![allow(non_snake_case)]
use serde::Deserialize;
// MongoDB Models

#[derive(Deserialize, Debug)]
pub struct UserModel {
    pub username: String,
    #[serde(rename = "discordID")] pub discord_id: i64,
    #[serde(rename = "gameID")] pub game_id: i64,
    pub avatar: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct LeaderboardModel {
    name: String,
    hash: String,
    
}

// Server Models
#[derive(Deserialize, Debug)]
pub struct ScoreModel {
    pub hash: String,
    pub difficulty: i32,
    pub characteristic: String,

    pub id: i64,
    pub multipliedScore: i64,
    pub modifiedScore: i64,
    pub accuracy: f32,
    pub misses: i32,
    pub badCuts: i32,
    pub fullCombo: bool,
    pub modifiers: String
}

#[derive(Deserialize, Debug)]
pub struct GetLeaderboardModel {
    userID: i32,
    diff: i32,
    char: String
}


// Getting Scores Query
#[derive(Deserialize, Debug)]
pub struct ScoresQueryModel {
    #[serde(rename = "char")] pub characteristic: String,
    #[serde(rename = "diff")] pub difficulty: String,
    pub sort: String,
    pub limit: i32,
    pub page: i32,
    #[serde(rename = "id")] pub user_id: i64,
}

#[derive(Deserialize, Debug)]
pub struct UserLoginModel {
    #[serde(rename = "id")] pub user_id: i64
}