#[derive(Clone, PartialEq, Eq)]
pub struct Proof {
    pub yt_id: Option<String>,
    pub url: String,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::api::*;
    use async_trait::async_trait;
    use axum_session_auth::Authentication;
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Serialize};
    use sqlx::{PgPool, types::BitVec};

    pub trait GetUser {
        #[allow(async_fn_in_trait)]
        async fn get_with_passhash(id: i64, pool: &PgPool) -> Option<(Self, UserPasshash)>
        where
            Self: Sized;
        #[allow(async_fn_in_trait)]
        async fn get(id: i64, pool: &PgPool) -> Option<Self>
        where
            Self: Sized;
        #[allow(async_fn_in_trait)]
        async fn get_from_username_with_passhash(name: String, pool: &PgPool) -> Option<(Self, UserPasshash)>
        where
            Self: Sized;
        #[allow(async_fn_in_trait)]
        async fn get_from_username(name: String, pool: &PgPool) -> Option<Self>
        where
            Self: Sized;
    }

    impl GetUser for User {
        async fn get_with_passhash(id: i64, pool: &PgPool) -> Option<(Self, UserPasshash)> {
            let pg_user = sqlx::query_as::<_, PgUser>("SELECT * FROM \"user\" WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .ok()?;
            let pg_user_ranks = sqlx::query_as::<_, Rank>(
                r#"SELECT patch, layout, category, title, rank, rating, percentage, created_at, updated_at
                    FROM rank
                    WHERE user_id = $1;
                "#,
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(pg_user.into_user(Some(pg_user_ranks)))
        }

        async fn get(id: i64, pool: &PgPool) -> Option<Self> {
            User::get_with_passhash(id, pool).await.map(|(user, _)| user)
        }

        async fn get_from_username_with_passhash(name: String, pool: &PgPool) -> Option<(Self, UserPasshash)> {
            let pg_user = sqlx::query_as::<_, PgUser>(
                r#"SELECT id, "name", password, pfp, bio, permission, created_at FROM "user" WHERE "name" = $1;"#,
            )
            .bind(name)
            .fetch_one(pool)
            .await
            .ok()?;

            let pg_user_ranks = sqlx::query_as::<_, Rank>(
                r#"SELECT patch, layout, category, title, rank, rating, percentage, created_at, updated_at
                    FROM rank
                    WHERE user_id = $1;
                "#,
            )
            .bind(pg_user.id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(pg_user.into_user(Some(pg_user_ranks)))
        }

        async fn get_from_username(name: String, pool: &PgPool) -> Option<Self> {
            User::get_from_username_with_passhash(name, pool)
                .await
                .map(|(user, _)| user)
        }
    }

    #[async_trait]
    impl Authentication<User, i64, PgPool> for User {
        async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
            let pool = pool.unwrap();

            User::get(userid, pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }

    // Explicitly is not Serialize/Deserialize!
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct UserPasshash(pub String);

    #[derive(sqlx::FromRow)]
    pub struct UserId {
        pub id: i64,
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct PgUser {
        pub id: i64,
        pub name: String,
        pub password: String,
        pub pfp: String,
        pub bio: Option<String>,
        pub permission: BitVec,
        pub created_at: DateTime<Local>,
    }

    impl PgUser {
        pub fn into_user(self, pg_user_ranks: Option<Vec<Rank>>) -> (User, UserPasshash) {
            (
                User {
                    id: self.id,
                    username: self.name,
                    bio: self.bio,
                    permissions: self.permission,
                    ranks: pg_user_ranks.unwrap_or_default(),
                    pfp: self.pfp,
                },
                UserPasshash(self.password),
            )
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct YtVidJson {
        pub kind: String,
        pub etag: String,
        pub id: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct YtPageInfo {
        pub total_results: i32,
        pub results_per_page: i32,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct YtJson {
        pub kind: String,
        pub etag: String,
        pub items: Vec<YtVidJson>,
        pub page_info: YtPageInfo,
    }

    #[derive(Deserialize)]
    pub struct AuthRes {
        pub access_token: String,
        pub token_type: String,
        pub expires_in: i64,
        pub refresh_token: String,
        pub scope: String,
    }
}
