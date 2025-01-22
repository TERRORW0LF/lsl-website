use chrono::{DateTime, Local};
use http::{header::CACHE_CONTROL, HeaderValue};
use leptos::prelude::{expect_context, server, server_fn::codec::GetUrl, ServerFnError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

use super::auth::User;

#[derive(Clone, Debug, Error, EnumString, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Unauthorized")]
    #[strum(to_string = "Unauthorized")]
    Unauthorized,
    #[error("Unauthenticated")]
    #[strum(to_string = "Unauthenticated")]
    Unauthenticated,
    #[error("Invalid Credentials")]
    #[strum(to_string = "Invalid Credentials")]
    InvalidCredentials,
    #[error("Invalid Section")]
    #[strum(to_string = "Invalid Section")]
    InvalidSection,
    #[error("Invalid YouTube ID")]
    #[strum(to_string = "Invalid YouTube ID")]
    InvalidYtId,
    #[error("Already Exists")]
    #[strum(to_string = "Already Exists")]
    AlreadyExists,
    #[error("Not Found")]
    #[strum(to_string = "Not Found")]
    NotFound,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RunFilters {
    pub before: Option<DateTime<Local>>,
    pub after: Option<DateTime<Local>>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub map: Option<String>,
    pub faster: Option<Decimal>,
    pub slower: Option<Decimal>,
    pub sort: String,
    pub ascending: bool,
}

impl Default for RunFilters {
    fn default() -> Self {
        Self {
            before: None,
            after: None,
            layout: None,
            category: None,
            map: None,
            faster: None,
            slower: None,
            sort: String::from("created_at"),
            ascending: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Run {
    pub id: i32,
    pub section_id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub user_id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub time: Decimal,
    pub proof: String,
    pub yt_id: Option<String>,
    pub verified: bool,
    pub is_pb: bool,
    pub is_wr: bool,
    pub created_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(no_pg_array))]
pub struct PartialRun {
    pub id: i32,
    pub section_id: i32,
    pub user_id: i64,
    pub username: String,
    pub time: Decimal,
    pub proof: String,
    pub yt_id: Option<String>,
    pub verified: bool,
    pub is_pb: bool,
    pub is_wr: bool,
    pub created_at: DateTime<Local>,
}

// WARNING: Absolutely horrid hack to make query_as function work with array_agg
// Probably destroys type safety, make sure to always double check queries
#[cfg(feature = "ssr")]
impl sqlx::postgres::PgHasArrayType for PartialRun {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_record")
    }
    fn array_compatible(_ty: &sqlx::postgres::PgTypeInfo) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MapRuns {
    pub id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub runs: Vec<PartialRun>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Map {
    #[cfg_attr(feature = "ssr", sqlx(rename = "map"))]
    pub name: String,
    pub code: String,
}

#[server(GetRunsId, prefix="/api", endpoint="runs/id", input=GetUrl)]
pub async fn get_runs_id(id: i32) -> Result<MapRuns, ServerFnError<ApiError>> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, MapRuns>(
        r#"SELECT s.id, s.patch, s.layout, s.category, s.map,
                COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, u.id, u."name", r.time,
                        r.proof, r.yt_id, r.verified, r.is_pb, r.is_wr, r.created_at)
                    ORDER BY r.created_at ASC)
                    FILTER(WHERE r.id IS NOT NULL), '{NULL}'), '{}') AS runs
            FROM section s
            LEFT JOIN run r ON section_id = s.id
            LEFT JOIN "user" u ON user_id = u.id
            WHERE s.id = $1
            GROUP BY s.id, patch, layout, category, map"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::InvalidSection)?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
    Ok(runs)
}

#[server(GetRunsCategory, prefix="/api", endpoint="runs/category", input=GetUrl)]
pub async fn get_runs_category(
    patch: String,
    layout: String,
    category: String,
) -> Result<Vec<MapRuns>, ServerFnError<ApiError>> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, MapRuns>(
        r#"SELECT s.id, patch, layout, category, map,
                COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, r.user_id, u."name", r.time,
                r.proof, r.yt_id, r.verified, r.is_pb, r.is_wr, r.created_at)
                    ORDER BY r.created_at ASC) 
                    FILTER(WHERE r.id IS NOT NULL), '{NULL}'), '{}') AS runs
            FROM section s
            LEFT JOIN run r ON section_id = s.id
            LEFT JOIN "user" u ON user_id = u.id
            WHERE patch = $1 AND layout = $2 AND category = $3
            GROUP BY s.id, patch, layout, category, map"#,
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await
    .map_err(|_| ServerFnError::ServerError("Database lookup failed".to_string()))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
    Ok(runs)
}

#[server(GetRunsLatest, prefix="/api", endpoint="runs/latest", input=GetUrl)]
pub async fn get_runs_latest(offset: i32) -> Result<Vec<Run>, ServerFnError<ApiError>> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, Run>(
        r#"SELECT run.id, run.created_at, section_id, patch, layout, 
                    category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
                FROM run
                INNER JOIN section ON section_id = section.id
                INNER JOIN "user" ON user_id = u.id
                ORDER BY run.created_at DESC
                LIMIT 50 OFFSET $1"#,
    )
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| ServerFnError::ServerError("Database lookup failed".to_string()))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=300"));
    Ok(runs)
}

#[server(GetRunsUser, prefix="/api", endpoint="runs/user", input=GetUrl)]
pub async fn get_runs_user(
    user_id: i64,
    filter: RunFilters,
    offset: i32,
) -> Result<Vec<Run>, ServerFnError<ApiError>> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, Run>(
        r#"SELECT run.id, run.created_at, section_id, patch, layout, 
                    category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
                FROM run
                INNER JOIN section s ON section_id = s.id
                INNER JOIN "user" u ON user_id = u.id
                WHERE user_id = $1
                ORDER BY run.created_at DESC
                LIMIT 50 OFFSET $2"#,
    )
    .bind(user_id)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError("Database lookup failed".to_string()))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=300"));
    Ok(runs)
}

#[server(GetMaps, prefix="/api", endpoint="maps", input=GetUrl)]
pub async fn get_maps(
    patch: String,
    layout: String,
    category: String,
) -> Result<Vec<Map>, ServerFnError<ApiError>> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let maps = sqlx::query_as::<_, Map>(
        r#"SELECT map, code
        FROM section
        WHERE patch=$1 AND layout=$2 AND category=$3"#,
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await
    .map_err(|_| ServerFnError::ServerError("Database lookup failed".to_string()))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
    Ok(maps)
}

#[server(GetUser, prefix="/api", endpoint="user/get", input=GetUrl)]
pub async fn get_user(id: i64) -> Result<User, ServerFnError<ApiError>> {
    let pool = crate::server::auth::ssr::pool()?;
    Ok(User::get(id, &pool).await.ok_or(ApiError::NotFound)?)
}
