use chrono::{Local, TimeDelta};
use http::HeaderValue;
use leptos::prelude::{server, server_fn::codec::PostUrl};
use rust_decimal::Decimal;
use server_fn::codec::{GetUrl, MultipartData, MultipartFormData};
use types::api::*;

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use argon2::{
        Argon2, PasswordHash, PasswordVerifier,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    pub use axum_session_auth::{Authentication, HasPermission};
    pub use axum_session_sqlx::SessionPgPool;
    pub use leptos::prelude::{server, use_context};
    use oauth2::basic::BasicClient;
    pub use sqlx::{
        PgPool,
        postgres::{PgConnectOptions, PgPoolOptions},
    };
    pub use std::env;
    pub use types::{api::*, internal::*, leptos::AuthSession};

    pub async fn connect_to_database() -> PgPool {
        let connect_opts = PgConnectOptions::new()
            .database(&std::env::var("PG_DB").unwrap())
            .username(&std::env::var("PG_USER").unwrap())
            .password(&std::env::var("PG_PASS").unwrap())
            .host(&std::env::var("PG_HOST").unwrap())
            .port(std::env::var("PG_PORT").unwrap().parse::<u16>().unwrap());

        PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_opts)
            .await
            .unwrap()
    }

    pub fn pool() -> Result<PgPool, ApiError> {
        use_context::<PgPool>().ok_or(ApiError::ServerError("Pool missing.".into()))
    }

    pub fn auth() -> Result<AuthSession, ApiError> {
        use_context::<AuthSession>().ok_or(ApiError::ServerError("Auth session missing.".into()))
    }

    pub fn oauth() -> Result<BasicClient, ApiError> {
        use_context::<BasicClient>().ok_or(ApiError::ServerError("OAuth client missing.".into()))
    }

    pub fn hash_password(password: &String) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(v) => Ok(v.to_string()),
            Err(_) => Err(ApiError::ServerError(
                "Signup failed: Failed to hash password".into(),
            )),
        }
    }

    pub fn verify_password(pass_hash: &String, password: &String) -> Result<(), ApiError> {
        let pwd_parsed = PasswordHash::new(pass_hash)
            .map_err(|_| ApiError::ServerError("Login failed: Failed to hash password".into()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &pwd_parsed)
            .or(Err(ApiError::InvalidCredentials))
    }

    pub fn check_password(password: &String) -> bool {
        password.len() >= 8 && password.len() <= 256
    }

    pub fn check_username(username: &String) -> bool {
        username.len() >= 2
            && username.len() <= 32
            && username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    }
}

#[server(GetCurrentUser, prefix="/api", endpoint="user/@me/get", input=PostUrl)]
pub async fn get_current_user() -> Result<User, ApiError> {
    use self::ssr::*;
    auth()?.current_user.ok_or(ApiError::Unauthenticated)
}

#[server(Register, prefix="/api", endpoint="user/register", input=PostUrl)]
pub async fn register(
    username: String,
    password: String,
    password_confirm: String,
    remember: Option<String>,
) -> Result<(), ApiError> {
    use self::ssr::*;

    if !check_password(&password) || !check_username(&username) {
        Err(ApiError::InvalidCredentials)?;
    }
    if password != password_confirm {
        Err(ApiError::InvalidCredentials)?;
    }

    let pool = pool()?;
    let auth = auth()?;

    let pwd_hash = hash_password(&password)?;

    sqlx::query("INSERT INTO \"user\" (name, password) VALUES ($1, $2);")
        .bind(username.clone())
        .bind(pwd_hash)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::AlreadyExists)?;

    let user = User::get_from_username(username, &pool)
        .await
        .ok_or_else(|| ApiError::ServerError("Signup failed: User does not exist.".into()))?;

    let _ = sqlx::query(
        r#"INSERT INTO permission (user_id, token) 
            VALUES ($1, $2), ($1, $3), ($1, $4), ($1, $5);"#,
    )
    .bind(user.id)
    .bind(Permissions::View)
    .bind(Permissions::Submit)
    .bind(Permissions::Trusted)
    .bind(Permissions::Delete)
    .execute(&pool)
    .await;

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
) -> Result<(), ApiError> {
    use self::ssr::*;

    let pool = pool()?;
    let auth = auth()?;

    let (user, UserPasshash(expected_passhash)) =
        User::get_from_username_with_passhash(username, &pool)
            .await
            .ok_or(ApiError::InvalidCredentials)?;
    verify_password(&expected_passhash, &password)?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());
    match HeaderValue::from_str(&format!("/{}", redirect.unwrap_or(String::new()))) {
        Ok(r) => leptos_axum::redirect(r.to_str().unwrap_or("/")),
        Err(_) => leptos_axum::redirect("/"),
    };
    Ok(())
}

#[server(UpdateCreds, prefix="/api", endpoint="user/update/credentials", input=PostUrl)]
pub async fn update_creds(
    username: Option<String>,
    password: Option<PasswordUpdate>,
    redirect: Option<String>,
) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    if !auth.current_user.is_some() {
        Err(ApiError::Unauthenticated)?;
    }
    let curr_user = auth.current_user.as_ref().unwrap();
    let pool = pool()?;

    if let Some(name) = username {
        if !check_username(&name) {
            Err(ApiError::InvalidCredentials)?;
        }
        sqlx::query(
            r#"UPDATE "user"
            SET name = $1
            WHERE id = $2;"#,
        )
        .bind(name)
        .bind(curr_user.id)
        .execute(&pool)
        .await
        .or(Err(ApiError::AlreadyExists))?;
        auth.cache_clear_user(curr_user.id);
    }
    if let Some(pw) = password {
        let (_, UserPasshash(expected_passhash)) =
            User::get_from_username_with_passhash(curr_user.username.clone(), &pool)
                .await
                .ok_or(ApiError::InvalidCredentials)?;
        verify_password(&expected_passhash, &pw.old)?;
        if !check_password(&pw.new) {
            Err(ApiError::InvalidCredentials)?;
        }
        let pwd_hash = hash_password(&pw.new)?;

        sqlx::query(
            r#"UPDATE "user"
            SET password = $1
            WHERE id = $2;"#,
        )
        .bind(pwd_hash)
        .bind(curr_user.id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;
        auth.cache_clear_user(curr_user.id);
    }
    if let Some(red) = redirect {
        if let Some(re) = HeaderValue::from_str(&format!("/{}", red)).ok() {
            leptos_axum::redirect(re.to_str().unwrap_or("/"));
        };
    }
    Ok(())
}

#[server(UpdateBio, prefix="/api", endpoint="user/update/bio", input=PostUrl)]
pub async fn update_bio(bio: Option<String>, redirect: Option<String>) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    if !auth.current_user.is_some() {
        Err(ApiError::Unauthenticated)?;
    }
    let curr_user = auth.current_user.as_ref().unwrap();
    let pool = pool()?;

    if bio.clone().is_some_and(|b| b.len() > 512) {
        Err(ApiError::InvalidCredentials)?;
    }
    sqlx::query(
        r#"UPDATE "user"
        SET bio = $1
        WHERE id = $2;"#,
    )
    .bind(bio)
    .bind(curr_user.id)
    .execute(&pool)
    .await
    .or(Err(ApiError::InvalidCredentials))?;
    auth.cache_clear_user(curr_user.id);

    if let Some(red) = redirect {
        if let Some(re) = HeaderValue::from_str(&format!("/{}", red)).ok() {
            leptos_axum::redirect(re.to_str().unwrap_or("/"));
        };
    }
    Ok(())
}

#[server(UpdatePfp, prefix="/api", endpoint="user/update/avatar", input=MultipartFormData)]
pub async fn update_pfp(data: MultipartData) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};
    use rand::{Rng, distributions::Alphanumeric, thread_rng};
    use std::fs::{File, remove_file};
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
        if pfp.name().unwrap_or_default() != "avatar" {
            return Err(ApiError::InvalidInput);
        }

        let name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        let file = File::options()
            .append(true)
            .create_new(true)
            .open(format!("target/site/cdn/users/{name}.jpg"));
        let res = match file {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                let mut jpg = false;
                while let Ok(Some(chunk)) = pfp.chunk().await {
                    let len = chunk.len();
                    count += len;
                    if !jpg {
                        jpg = true;
                        if !(chunk.len() > 2
                            && chunk[0] == 0xFF
                            && chunk[1] == 0xD8
                            && chunk[2] == 0xFF)
                        {
                            return Err(ApiError::InvalidInput);
                        }
                    }
                    if count > 4 * 1024 * 1024 {
                        return Err(ApiError::InvalidInput);
                    }
                    writer
                        .write_all(&chunk)
                        .map_err(|_| ApiError::ServerError("Failed to save file".into()))?;
                }
                writer
                    .flush()
                    .map_err(|_| ApiError::ServerError("Failed to save file".into()))?;
                sqlx::query(
                    r#"UPDATE "user"
                    SET pfp = $1
                    WHERE id = $2;"#,
                )
                .bind(name.clone())
                .bind(user.id)
                .execute(&pool)
                .await
                .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

                auth.cache_clear_user(user.id);
                Ok(())
            }
            Err(_) => Err(ApiError::ServerError("File creation failed".into())),
        };
        match res {
            Ok(_) => {
                let _ = remove_file(format!("../cdn/users/{}.jpg", user.pfp));
                Ok(())
            }
            Err(err) => {
                let _ = remove_file(format!("../cdn/users/{name}.jpg"));
                Err(err)
            }
        }
    } else {
        Err(ApiError::InvalidInput)
    }
}

#[server(Logout, prefix="/api", endpoint="user/logout", input=PostUrl)]
pub async fn logout() -> Result<(), ApiError> {
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
) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    let pool = pool()?;

    let u = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    if !u.has(&Permissions::Submit) {
        return Err(ApiError::Unauthorized);
    }

    let section_id = sqlx::query_as::<_, SectionId>(
        r#"SELECT id
        FROM section
        WHERE patch='2.13' AND layout=$1 AND category=$2 AND map=$3;"#,
    )
    .bind(&layout)
    .bind(&category)
    .bind(map)
    .fetch_one(&pool)
    .await
    .or(Err(ApiError::InvalidSection))?;

    let r = reqwest::get(format!(
        "https://www.googleapis.com/youtube/v3/videos?key={}&part=id&id={yt_id}",
        env!("YT_KEY")
    ))
    .await
    .map_err(|_| ApiError::ServerError("YT api request failed".into()))?;

    let v = r
        .json::<YtJson>()
        .await
        .map_err(|_| ApiError::ServerError("Failed to parse yt api response".into()))?;

    if v.page_info.total_results == 0 {
        Err(ApiError::InvalidYtId)
    } else {
        let proof = format!("https://youtube.com/watch?v={yt_id}");
        let _ = sqlx::query(
            r#"INSERT INTO run (section_id, user_id, time, proof, yt_id, verified)
                                    VALUES ($1, $2, $3, $4, $5, $6);"#,
        )
        .bind(section_id.id)
        .bind(u.id)
        .bind(time)
        .bind(proof)
        .bind(yt_id)
        .bind(u.has(&Permissions::Trusted))
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database insert failed".into()))?;

        leptos_axum::redirect(&format!("/leaderboard/map/{}", section_id.id));
        Ok(())
    }
}

#[server(Verify, prefix="/api", endpoint="runs/verify", input=PostUrl)]
pub async fn verify(id: i32) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    let pool = pool()?;

    let u = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    if !u.has(&Permissions::Verify) {
        return Err(ApiError::Unauthorized);
    }

    sqlx::query(
        r#"UPDATE run
        SET verified = TRUE
        WHERE id = $1;"#,
    )
    .bind(id)
    .execute(&pool)
    .await
    .or(Err(ApiError::NotFound))?;
    Ok(())
}

#[server(Delete, prefix="/api", endpoint="runs/delete", input=PostUrl)]
pub async fn delete(id: i32, redirect: Option<String>) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    let pool = pool()?;

    let u = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    if !u.has(&Permissions::Delete) {
        return Err(ApiError::Unauthorized);
    }

    let num = sqlx::query(
        r#"DELETE FROM run
        WHERE id = $1 AND user_id = $2 AND section_id >= 1093;"#,
    )
    .bind(id)
    .bind(u.id)
    .execute(&pool)
    .await;
    if num.is_ok_and(|r| r.rows_affected() != 0) {
        if let Some(red) = redirect {
            if let Some(re) = HeaderValue::from_str(&format!("/{}", red)).ok() {
                leptos_axum::redirect(re.to_str().unwrap_or("/"));
            };
        }
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

#[server(DiscordList, prefix="/api", endpoint="user/discord/list", input=PostUrl)]
pub async fn discord_list() -> Result<Vec<Discord>, ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    let user = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    let pool = pool()?;

    sqlx::query_as::<_, Discord>(
        r#"SELECT name, snowflake
        FROM discord
        WHERE user_id = $1
        LIMIT 5;"#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database lookup failed".into()))
}

#[server(DiscordAdd, prefix="/api", endpoint="user/discord/add", input=PostUrl)]
pub async fn discord_add() -> Result<(), ApiError> {
    use self::ssr::*;
    use oauth2::{CsrfToken, Scope};

    let auth = auth()?;
    if auth.current_user.is_none() {
        return Err(ApiError::Unauthenticated);
    }
    let oauth = oauth()?;

    let (auth_url, csrf_token) = oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".into()))
        .add_scope(Scope::new("role_connections.write".into()))
        .url();

    auth.session.set("csrf", csrf_token);
    leptos_axum::redirect(auth_url.as_ref());
    Ok(())
}

#[server(DiscordAuth, prefix="/api", endpoint="user/discord/auth", input=GetUrl)]
pub async fn discord_auth(code: String, state: String) -> Result<(), ApiError> {
    use self::ssr::*;
    use oauth2::{AuthorizationCode, CsrfToken, TokenResponse, reqwest::async_http_client};

    leptos_axum::redirect("/user/@me/dashboard");

    let auth = auth()?;
    let user = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    let oauth = oauth()?;
    let csrf = auth
        .session
        .get_remove::<CsrfToken>("csrf")
        .ok_or(ApiError::Unauthenticated)?;
    if *csrf.secret() != state {
        return Err(ApiError::InvalidCredentials);
    }
    let token = oauth
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|_| ApiError::ServerError("Token exchange failed".into()))?;

    // Fetch user data from discord
    let client = reqwest::Client::new();
    let discord_data: Discord = client
        // https://discord.com/developers/docs/resources/user#get-current-user
        .get("https://discordapp.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .map_err(|_| ApiError::ServerError("Discord fetch failed".into()))?
        .json::<Discord>()
        .await
        .map_err(|_| ApiError::ServerError("Discord reponse invalid".into()))?;
    let name = discord_data.name;
    let snowflake = discord_data.snowflake;

    let pool = pool()?;

    let discord = sqlx::query_as::<_, Discord>(
        r#"SELECT name, snowflake
        FROM discord
        WHERE user_id = $1
        LIMIT 5;"#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database lookup failed".into()))?;

    if discord.len() >= 5 {
        return Err(ApiError::AlreadyExists)?;
    }
    if discord.iter().any(|d| d.snowflake == snowflake) {
        sqlx::query(
            r#"UPDATE discord
            SET name = $1, access = $2, refresh = $3, expires_at = $4
            WHERE user_id = $5 AND snowflake = $6;"#,
        )
        .bind(name)
        .bind(token.access_token().secret())
        .bind(token.refresh_token().unwrap().secret())
        .bind(Local::now() + TimeDelta::seconds(token.expires_in().unwrap().as_secs() as i64))
        .bind(user.id)
        .bind(snowflake)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;
    } else {
        sqlx::query(
            r#"INSERT INTO discord (user_id, name, snowflake, access, refresh, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6);"#,
        )
        .bind(user.id)
        .bind(name)
        .bind(snowflake)
        .bind(token.access_token().secret())
        .bind(token.refresh_token().unwrap().secret())
        .bind(Local::now() + TimeDelta::seconds(token.expires_in().unwrap().as_secs() as i64))
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database insert failed".into()))?;
    }
    Ok(())
}

#[server(DiscordDelete, prefix="/api", endpoint="user/discord/delete", input=PostUrl)]
pub async fn discord_delete(snowflake: String) -> Result<(), ApiError> {
    use self::ssr::*;

    let auth = auth()?;
    let user = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    let pool = pool()?;

    let _ = sqlx::query(
        r#"DELETE FROM discord
        WHERE user_id = $1 AND snowflake = $2;"#,
    )
    .bind(user.id)
    .bind(snowflake)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database delete failed".to_string()))?;
    Ok(())
}
