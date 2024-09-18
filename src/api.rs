use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::{
        response::Json,
        extract::Query,
    };
    use serde::{ Serialize, Deserialize };

    #[derive(Deserialize)]
    pub struct FindUsers {
        username: String,
        limit: u8,
    }

    #[derive(Serialize)]
    pub struct PartialUser {
        id: u64,
        discord_id: Vec<u64>,
        username: String,
    }

    pub async fn find_users(user_slice: Query<FindUsers>) -> Json<Vec<PartialUser>> {
        let user = PartialUser { id: 0, discord_id: vec![0], username: "test".to_string() };
        Json(vec![user])
    }
}}
