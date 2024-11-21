use crate::server::api::ApiError;
use http::HeaderValue;
use leptos::prelude::{server, server_fn::codec::PostUrl, ServerFnError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub permissions: HashSet<String>,
    pub pfp: String,
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
            pfp: "default".into(),
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PasswordUpdate {
    old: String,
    new: String,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::server::api::ApiError;

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

    pub fn pool() -> Result<PgPool, ServerFnError<ApiError>> {
        use_context::<PgPool>().ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError<ApiError>> {
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".to_string()))
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
        pub pfp: String,
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
                    pfp: self.pfp,
                },
                UserPasshash(self.password),
            )
        }
    }

    pub fn verify_password(
        pass_hash: String,
        password: String,
    ) -> Result<(), ServerFnError<ApiError>> {
        let pwd_parsed = PasswordHash::new(&pass_hash).map_err(|_| {
            ServerFnError::ServerError("Login failed: Failed to hash password".to_string())
        })?;

        Argon2::default()
            .verify_password(password.as_bytes(), &pwd_parsed)
            .map_err(|_| ApiError::InvalidCredentials)?;
        Ok(())
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
) -> Result<(), ServerFnError<ApiError>> {
    use self::ssr::*;

    if password != password_confirm {
        return Err(ApiError::InvalidCredentials.into());
    }

    let pool = pool()?;
    let auth = auth()?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pwd_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(v) => v.to_string(),
        Err(_) => {
            return Err(ServerFnError::ServerError(
                "Signup failed: Failed to hash password".to_string(),
            ))
        }
    };

    sqlx::query("INSERT INTO \"user\" (name, password) VALUES ($1,$2)")
        .bind(username.clone())
        .bind(pwd_hash)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::AlreadyExists)?;

    let user = User::get_from_username(username, &pool)
        .await
        .ok_or_else(|| {
            ServerFnError::ServerError("Signup failed: User does not exist.".to_string())
        })?;

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
    redirect: Option<String>,
) -> Result<(), ServerFnError<ApiError>> {
    use self::ssr::*;

    let pool = pool()?;
    let auth = auth()?;

    let (user, UserPasshash(expected_passhash)) =
        User::get_from_username_with_passhash(username, &pool)
            .await
            .ok_or(ApiError::InvalidCredentials)?;
    verify_password(expected_passhash, password)?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());
    match HeaderValue::from_str(&format!("/{}", redirect.unwrap_or(String::new()))) {
        Ok(r) => leptos_axum::redirect(r.to_str().unwrap_or("/")),
        Err(_) => leptos_axum::redirect("/"),
    };
    Ok(())
}

#[server(Update, prefix="/api", endpoint="user/update", input=PostUrl)]
pub async fn update(
    username: Option<String>,
    password: Option<PasswordUpdate>,
) -> Result<(), ServerFnError<ApiError>> {
    use self::ssr::*;

    let auth = auth()?;
    if !auth.current_user.is_some() {
        Err(ApiError::Unauthenticated)?;
    }
    let curr_user = auth.current_user.as_ref().unwrap();
    let pool = pool()?;

    if let Some(name) = username {
        sqlx::query(
            r#"UPDATE "user"
            SET name = $1
            WHERE id = $2"#,
        )
        .bind(name)
        .bind(curr_user.id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::AlreadyExists)?;
        auth.cache_clear_user(curr_user.id);
    }
    if let Some(pw) = password {
        let (_, UserPasshash(expected_passhash)) =
            User::get_from_username_with_passhash(curr_user.username.clone(), &pool)
                .await
                .ok_or(ApiError::InvalidCredentials)?;
        verify_password(expected_passhash, pw.old)?;
        sqlx::query(
            r#"UPDATE "user"
            SET password = $1
            WHERE id = $2"#,
        )
        .bind(pw.new)
        .bind(curr_user.id)
        .execute(&pool)
        .await
        .map_err(|_| ServerFnError::<ApiError>::ServerError("Database update failed".into()))?;
        auth.cache_clear_user(curr_user.id);
    }
    Ok(())
}

#[server(Pfp, prefix="/api", endpoint="user/update", input=MultipartFormData)]
pub async fn pfp(data: MultipartData) -> Result<(), ServerFnError<ApiError>> {
    use crate::server::auth::ssr::{auth, pool};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::fs::{remove_file, File};
    use std::io::{BufWriter, Write};

    let auth = auth()?;
    let pool = pool()?;

    if !auth.current_user.is_some() {
        return Err(ApiError::Unauthenticated)?;
    }
    let user = auth.current_user.as_ref().unwrap();

    let mut data = data.into_inner().unwrap();
    let mut count = 0;

    if let Ok(Some(mut pfp)) = data.next_field().await {
        if pfp.name().unwrap_or_default() != "pfp" {
            return Err(ServerFnError::<ApiError>::Args(
                "Pfp must be only field".into(),
            ));
        }
        if pfp.file_name().unwrap_or_default().ends_with(".jpg") {
            return Err(ServerFnError::<ApiError>::Args("Must be jpg file".into()));
        }

        let name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        let file = File::options()
            .append(true)
            .create_new(true)
            .open(format!("../cdn/users/{name}.jpg"));
        match file {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                while let Ok(Some(chunk)) = pfp.chunk().await {
                    let len = chunk.len();
                    count += len;
                    if count > 1024 * 1024 {
                        return Err(ServerFnError::<ApiError>::Args("Too big".into()));
                    }
                    writer.write_all(&chunk).map_err(|_| {
                        ServerFnError::<ApiError>::ServerError("Failed to save file".into())
                    })?;
                }
                writer.flush().map_err(|_| {
                    ServerFnError::<ApiError>::ServerError("Failed to save file".into())
                })?;
                sqlx::query(
                    r#"UPDATE "user"
                    SET pfp = $1
                    WHERE id = $2"#,
                )
                .bind("")
                .bind("")
                .execute(&pool)
                .await
                .map_err(|_| {
                    let _ = remove_file(format!("../cdn/users/{name}.jpg"));
                    ServerFnError::<ApiError>::ServerError("Database update failed".into())
                })?;

                let _ = remove_file(format!("../cdn/users/{}.jpg", user.pfp));
                auth.cache_clear_user(user.id);
                Ok(())
            }
            Err(_) => Err(ServerFnError::<ApiError>::ServerError(
                "File creation failed".into(),
            )),
        }
    } else {
        Err(ServerFnError::<ApiError>::Args(
            "Pfp must be only field".into(),
        ))
    }
}

#[server(Logout, prefix="/api", endpoint="user/logout", input=PostUrl)]
pub async fn logout() -> Result<(), ServerFnError<ApiError>> {
    use self::ssr::*;

    let auth = auth()?;

    auth.logout_user();
    leptos_axum::redirect("/");

    Ok(())
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
struct SectionId {
    id: i32,
}

#[server(Submit, prefix="/api", endpoint="runs/submit", input=PostUrl)]
pub async fn submit(
    layout: String,
    category: String,
    map: String,
    time: Decimal,
    yt_id: String,
) -> Result<(), ServerFnError<ApiError>> {
    use self::ssr::*;

    let auth = auth()?;
    let pool = pool()?;

    let u = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    let section_id = sqlx::query_as::<_, SectionId>(
        r#"SELECT id
        FROM section
        WHERE patch='2.00' AND layout=$1 AND category=$2 AND map=$3"#,
    )
    .bind(layout)
    .bind(category)
    .bind(map)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::InvalidSection)?;

    let r = reqwest::get(format!(
        "https://www.googleapis.com/youtube/v3/videos?key={}&part=id&id={yt_id}",
        env!("YT_KEY")
    ))
    .await
    .map_err(|_| ServerFnError::ServerError("YT api request failed".to_string()))?;

    let v = r
        .json::<YtJson>()
        .await
        .map_err(|_| ServerFnError::ServerError("Failed to parse yt api response".to_string()))?;

    if v.page_info.total_results == 0 {
        Err(ApiError::InvalidYtId.into())
    } else {
        let proof = format!("https://youtube.com/watch?v={yt_id}");
        let res = sqlx::query(
            r#"INSERT INTO run (section_id, user_id, time, proof, yt_id, verified)
                                    VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(section_id.id)
        .bind(u.id)
        .bind(time)
        .bind(proof)
        .bind(yt_id)
        .bind(true)
        .execute(&pool)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerFnError::ServerError(
                "Database insert failed".to_string(),
            )),
        }
    }
}
