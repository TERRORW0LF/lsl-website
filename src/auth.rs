use leptos::prelude::{server, server_fn::codec::PostUrl, ServerFnError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub permissions: HashSet<String>,
}

// Explicitly is not Serialize/Deserialize!
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPasshash(String);

impl Default for User {
    fn default() -> Self {
        let permissions = HashSet::new();

        Self {
            id: -1,
            username: "Guest".into(),
            permissions,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtVidJson {
    kind: String,
    etag: String,
    id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtPageInfo {
    total_results: i32,
    results_per_page: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtJson {
    kind: String,
    etag: String,
    items: Vec<YtVidJson>,
    page_info: YtPageInfo,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::{User, UserPasshash};
    pub use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2, PasswordHash, PasswordVerifier,
    };
    pub use axum::async_trait;
    pub use axum_session_auth::{Authentication, HasPermission};
    pub use axum_session_sqlx::SessionPgPool;
    pub use leptos::prelude::{server, use_context, ServerFnError};
    use sqlx::types::chrono::{DateTime, Local};
    pub use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        PgPool,
    };
    use std::collections::HashSet;
    pub use std::env;
    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;

    pub async fn connect_to_database() -> PgPool {
        let mut connect_opts = PgConnectOptions::new();
        connect_opts = connect_opts
            .database(env!("PG_DB"))
            .username(env!("PG_USER"))
            .password(env!("PG_PASS"))
            .host(env!("PG_HOST"))
            .port(env!("PG_PORT").parse::<u16>().unwrap());

        PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_opts)
            .await
            .unwrap()
    }

    pub fn pool() -> Result<PgPool, ServerFnError> {
        use_context::<PgPool>().ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
    }

    impl User {
        pub async fn get_with_passhash(id: i64, pool: &PgPool) -> Option<(Self, UserPasshash)> {
            let pg_user = sqlx::query_as::<_, PgUser>("SELECT * FROM \"user\" WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .ok()?;

            //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
            let pg_user_perms = sqlx::query_as::<_, PgPermissionTokens>(
                "SELECT token FROM permission WHERE user_id = $1;",
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(pg_user.into_user(Some(pg_user_perms)))
        }

        pub async fn get(id: i64, pool: &PgPool) -> Option<Self> {
            User::get_with_passhash(id, pool)
                .await
                .map(|(user, _)| user)
        }

        pub async fn get_from_username_with_passhash(
            name: String,
            pool: &PgPool,
        ) -> Option<(Self, UserPasshash)> {
            let pg_user = sqlx::query_as::<_, PgUser>("SELECT * FROM \"user\" WHERE \"name\" = $1")
                .bind(name)
                .fetch_one(pool)
                .await
                .ok()?;

            //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
            let pg_user_perms = sqlx::query_as::<_, PgPermissionTokens>(
                "SELECT token FROM permission WHERE user_id = $1;",
            )
            .bind(pg_user.id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(pg_user.into_user(Some(pg_user_perms)))
        }

        pub async fn get_from_username(name: String, pool: &PgPool) -> Option<Self> {
            User::get_from_username_with_passhash(name, pool)
                .await
                .map(|(user, _)| user)
        }
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct PgPermissionTokens {
        pub token: String,
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

    #[async_trait]
    impl HasPermission<PgPool> for User {
        async fn has(&self, perm: &str, _pool: &Option<&PgPool>) -> bool {
            self.permissions.contains(perm)
        }
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct PgUser {
        pub id: i64,
        pub name: String,
        pub created_at: DateTime<Local>,
        pub password: String,
    }

    impl PgUser {
        pub fn into_user(
            self,
            pg_user_perms: Option<Vec<PgPermissionTokens>>,
        ) -> (User, UserPasshash) {
            (
                User {
                    id: self.id,
                    username: self.name,
                    permissions: if let Some(user_perms) = pg_user_perms {
                        user_perms
                            .into_iter()
                            .map(|x| x.token)
                            .collect::<HashSet<String>>()
                    } else {
                        HashSet::<String>::new()
                    },
                },
                UserPasshash(self.password),
            )
        }
    }
}

#[server(GetUser, prefix="/api", endpoint="user/get", input=PostUrl)]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    use self::ssr::*;

    let auth = auth()?;

    Ok(auth.current_user)
}

#[server(Register, prefix="/api", endpoint="user/register", input=PostUrl)]
pub async fn register(
    username: String,
    password: String,
    password_confirm: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    use self::ssr::*;

    if password != password_confirm {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }

    let pool = pool()?;
    let auth = auth()?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pwd_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(v) => v.to_string(),
        Err(_) => return Err(ServerFnError::new("Signup failed: Failed to hash password")),
    };

    sqlx::query("INSERT INTO \"user\" (name, password) VALUES ($1,$2)")
        .bind(username.clone())
        .bind(pwd_hash)
        .execute(&pool)
        .await?;

    let user = User::get_from_username(username, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("Signup failed: User does not exist."))?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());

    leptos_axum::redirect("/");

    Ok(())
}

#[server(Login, prefix="/api", endpoint="user/login", input=PostUrl)]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = pool()?;
    let auth = auth()?;

    let (user, UserPasshash(expected_passhash)) =
        User::get_from_username_with_passhash(username, &pool)
            .await
            .ok_or_else(|| ServerFnError::new("User does not exist."))?;
    let pwd_parsed = match PasswordHash::new(&expected_passhash) {
        Ok(v) => v,
        Err(_) => return Err(ServerFnError::new("Login failed: Failed to hash password")),
    };

    match Argon2::default().verify_password(password.as_bytes(), &pwd_parsed) {
        Ok(_) => {
            auth.login_user(user.id);
            auth.remember_user(remember.is_some());
            leptos_axum::redirect("/");
            Ok(())
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}

#[server(Logout, prefix="/api", endpoint="user/logout", input=PostUrl)]
pub async fn logout() -> Result<(), ServerFnError> {
    use self::ssr::*;

    let auth = auth()?;

    auth.logout_user();
    leptos_axum::redirect("/");

    Ok(())
}

#[server(Submit, prefix="/api", endpoint="runs/submit", input=PostUrl)]
pub async fn submit(section_id: i32, time: Decimal, yt_id: String) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let auth = auth()?;

    match auth.current_user {
        Some(u) => {
            match reqwest::get(format!(
                "https://www.googleapis.com/youtube/v3/videos?part=id&id={yt_id}"
            ))
            .await
            {
                Ok(r) => match r.json::<YtJson>().await {
                    Ok(v) => {
                        if v.page_info.total_results == 0 {
                            Err(ServerFnError::ServerError("Video not found".to_string()))
                        } else {
                            let proof = format!("https://youtube.com/watch?v={yt_id}");
                            let pool = pool()?;
                            let res = sqlx::query(
                                r#"INSERT INTO run (section_id, user_id, time, proof, yt_id, verified)
                                VALUES ($1, $2, $3, $4, $5, $6)"#
                            ).bind(section_id).bind(u.id).bind(time).bind(proof).bind(yt_id).bind(true).execute(&pool).await;

                            match res {
                                Ok(_) => Ok(()),
                                Err(_) => Err(ServerFnError::ServerError(
                                    "Database insert failed".to_string(),
                                )),
                            }
                        }
                    }
                    Err(_) => Err(ServerFnError::ServerError(
                        "Failed to parse yt api response".to_string(),
                    )),
                },
                Err(_) => Err(ServerFnError::ServerError(
                    "YT api request failed".to_string(),
                )),
            }
        }
        None => Err(ServerFnError::ServerError("Not logged in".to_string())),
    }
}
