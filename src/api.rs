use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::{
        response::Json,
        extract::Query,
    };
    use serde::{ Serialize, Deserialize };

    #[derive(Deserialize)]
    struct FindUsers {
        username: String,
        limit: u8,
    }

    #[derive(Serialize)]
    struct PartialUser {
        id: u64,
        discord_id: Vec<u64>,
        username: String,
    }

    pub async fn find_users(user_slice: Query<FindUsers>) -> Json<Vec<PartialUser>> {
        
    }
}}