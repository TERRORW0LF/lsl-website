use super::auth::User;
use chrono::{DateTime, Local};
use http::{header::CACHE_CONTROL, HeaderValue};
use leptos::prelude::{
    expect_context, server, server_fn::codec::GetUrl, FromServerFnError, ServerFnErrorErr,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use strum::{Display, EnumString};
use thiserror::Error;

#[derive(Clone, Debug, Error, EnumString, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Unauthorized")]
    #[strum(to_string = "Unauthorized")]
    Unauthorized,
    #[error("Unauthenticated")]
    #[strum(to_string = "Unauthenticated")]
    Unauthenticated,
    #[error("Invalid Input")]
    #[strum(to_string = "Invalid Input")]
    InvalidInput,
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
    #[error("Client Error: {0}")]
    ClientError(String),
    #[error("Server Error: {0}")]
    ServerError(String),
}

impl FromServerFnError for ApiError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        use ServerFnErrorErr::*;

        match value {
            UnsupportedRequestMethod(v) | Request(v) | Deserialization(v) | Serialization(v) => {
                ApiError::ClientError(v)
            }
            Registration(v) | MiddlewareError(v) | ServerError(v) | Args(v) | MissingArg(v)
            | Response(v) => ApiError::ServerError(v),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(type_name = "title"))]
pub enum Title {
    #[strum(to_string = "No Title")]
    None,
    #[strum(to_string = "Surfer")]
    Surfer,
    #[strum(to_string = "Super Surfer")]
    SuperSurfer,
    #[strum(to_string = "Epic Surfer")]
    EpicSurfer,
    #[strum(to_string = "Legendary Surfer")]
    LegendarySurfer,
    #[strum(to_string = "Mythic Surfer")]
    MythicSurfer,
    #[strum(to_string = "Rank 1")]
    TopOne,
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

#[cfg(feature = "ssr")]
impl sqlx::postgres::PgHasArrayType for PartialRanking {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_record")
    }
    fn array_compatible(_ty: &sqlx::postgres::PgTypeInfo) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SectionRuns {
    pub id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub runs: Vec<PartialRun>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Ranking {
    pub id: i32,
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub user_id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(no_pg_array))]
pub struct PartialRanking {
    pub id: i32,
    pub user_id: i64,
    pub username: String,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ComboRanking {
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub rankings: Vec<PartialRanking>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Map {
    #[cfg_attr(feature = "ssr", sqlx(rename = "map"))]
    pub name: String,
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Activity {
    pub id: i32,
    pub user_id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub rank_id: Option<i32>,
    pub patch: Option<String>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub title_old: Option<Title>,
    pub title_new: Option<Title>,
    pub rank_old: Option<i32>,
    pub rank_new: Option<i32>,
    pub created_at: DateTime<Local>,
}

#[server(GetRunsId, prefix="/api", endpoint="runs/id", input=GetUrl)]
pub async fn get_runs_id(id: i32) -> Result<SectionRuns, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, SectionRuns>(
        r#"SELECT s.id, s.patch, s.layout, s.category, s.map,
            COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, u.id, u."name", r.time,
                r.proof, r.yt_id, r.verified, r.is_pb, r.is_wr, r.created_at)
            ORDER BY r.created_at ASC)
            FILTER(WHERE r.id IS NOT NULL), '{NULL}'), '{}') AS runs
        FROM section s
        LEFT JOIN run r ON section_id = s.id
        LEFT JOIN "user" u ON user_id = u.id
        WHERE s.id = $1
        GROUP BY s.id, patch, layout, category, map;"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .or(Err(ApiError::InvalidSection))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
    Ok(runs)
}

#[server(GetRunsCategory, prefix="/api", endpoint="runs/category", input=GetUrl)]
pub async fn get_runs_category(
    patch: String,
    layout: String,
    category: String,
) -> Result<Vec<SectionRuns>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let runs = sqlx::query_as::<_, SectionRuns>(
        r#"SELECT s.id, patch, layout, category, map,
            COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, r.user_id, u."name", r.time,
                r.proof, r.yt_id, r.verified, r.is_pb, r.is_wr, r.created_at)
            ORDER BY r.created_at ASC) 
            FILTER(WHERE r.id IS NOT NULL), '{NULL}'), '{}') AS runs
        FROM section s
        LEFT JOIN run r ON section_id = s.id
        LEFT JOIN "user" u ON user_id = u.id
        WHERE patch = $1 AND layout = $2 AND category = $3
        GROUP BY s.id, patch, layout, category, map;"#,
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError("Database lookup failed".into())))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
    Ok(runs)
}

#[server(GetRunsLatest, prefix="/api", endpoint="runs/latest", input=GetUrl)]
pub async fn get_runs_latest(offset: i32) -> Result<Vec<Run>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    sqlx::query_as::<_, Run>(
        r#"SELECT run.id, run.created_at, section_id, patch, layout, 
            category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
        FROM run
        INNER JOIN section s ON section_id = s.id
        INNER JOIN "user" u ON user_id = u.id
        ORDER BY run.created_at DESC
        LIMIT 50 OFFSET $1;"#,
    )
    .bind(offset)
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError("Database lookup failed".into())))
}

#[server(GetRunsUser, prefix="/api", endpoint="runs/user", input=GetUrl)]
pub async fn get_runs_user(
    user_id: i64,
    filter: RunFilters,
    offset: i32,
) -> Result<Vec<Run>, ApiError> {
    use sqlx::{Postgres, QueryBuilder};

    let pool = crate::server::auth::ssr::pool()?;
    let mut query = QueryBuilder::<Postgres>::new(
        r#"SELECT run.id, run.created_at, section_id, patch, layout, 
            category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
        FROM run
        INNER JOIN section s ON section_id = s.id
        INNER JOIN "user" u ON user_id = u.id 
        WHERE patch = '2.13'"#,
    );
    query.push(" AND user_id = ").push_bind(user_id);
    if let Some(before) = filter.before {
        query.push(" AND run.created_at <= ").push_bind(before);
    }
    if let Some(after) = filter.after {
        query.push(" AND run.created_at >= ").push_bind(after);
    }
    if let Some(layout) = filter.layout {
        query.push(" AND layout = ").push_bind(layout);
    }
    if let Some(category) = filter.category {
        query.push(" AND category = ").push_bind(category);
    }
    if let Some(map) = filter.map {
        query.push(" AND map = ").push_bind(map);
    }
    if let Some(faster) = filter.faster {
        query.push(" AND time <= ").push_bind(faster);
    }
    if let Some(slower) = filter.slower {
        query.push(" AND time >= ").push_bind(slower);
    }
    query
        .push(" ORDER BY ")
        .push(match filter.sort.as_str() {
            "section" => "layout, category, map",
            "time" => "time",
            _ => "run.created_at",
        })
        .push(match filter.ascending {
            true => " ASC",
            false => " DESC",
        })
        .push(" LIMIT 50 OFFSET ")
        .push_bind(offset)
        .push(";");
    query
        .build_query_as::<Run>()
        .fetch_all(&pool)
        .await
        .or(Err(ApiError::ServerError(
            "Database lookup failed".to_string(),
        )))
}

#[server(GetMaps, prefix="/api", endpoint="maps", input=GetUrl)]
pub async fn get_maps() -> Result<Vec<Map>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let maps = sqlx::query_as::<_, Map>(
        r#"SELECT map, code
        FROM section
        WHERE patch='2.13' AND layout='1' AND category='Standard';"#,
    )
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError(
        "Database lookup failed".to_string(),
    )))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
    Ok(maps)
}

#[server(GetUser, prefix="/api", endpoint="user/get", input=GetUrl)]
pub async fn get_user(id: i64) -> Result<User, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    User::get(id, &pool).await.ok_or(ApiError::NotFound)
}

#[server(GetRankings, prefix="/api", endpoint="ranking", input=GetUrl)]
pub async fn get_rankings(
    patch: String,
    layout: Option<String>,
    category: Option<String>,
) -> Result<Vec<Ranking>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let rankings = sqlx::query_as::<_, Ranking>(
        r#"SELECT r.id, r.patch, r.layout, r.category, r.user_id, 
            u.name, r.title, r.rank, r.rating, r.created_at, r.updated_at
        FROM rank r
        JOIN "user" u ON user_id = u.id
        WHERE patch = $1 AND layout IS NOT DISTINCT FROM $2 AND category IS NOT DISTINCT FROM $3
        ORDER BY rank ASC;"#,
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await
    .or_else(|e| {
        leptos::logging::log!("{e:?}");
        Err(ApiError::NotFound)
    })?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
    Ok(rankings)
}

#[server(GetRankingsUser, prefix="/api", endpoint="ranking/user", input=GetUrl)]
pub async fn get_rankings_user(id: i64) -> Result<Vec<Ranking>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;

    sqlx::query_as::<_, Ranking>(
        r#"SELECT r.id, r.patch, r.layout, r.category, r.user_id, 
            u.name, r.title, r.rank, r.rating, r.created_at, r.updated_at
        FROM rank r
        JOIN "user" u ON user_id = u.id
        WHERE user_id = $1;"#,
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database lookup failed".into()))
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
struct UserId {
    id: i64,
}

#[server(GetPotd, prefix="/api", endpoint="user/get/potd", input=GetUrl)]
pub async fn get_potd() -> Result<User, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    let id = sqlx::query_as::<_, UserId>(
        r#"SELECT id
        FROM "user"
        WHERE bio IS NOT NULL
        ORDER BY random()
        LIMIT 1;"#,
    )
    .fetch_one(&pool)
    .await
    .or(Err(ApiError::NotFound))?;
    User::get(id.id, &pool).await.ok_or(ApiError::NotFound)
}

#[server(GetActivity, prefix="/api", endpoint="activity/get", input=GetUrl)]
pub async fn get_activity_latest(offset: i32) -> Result<Vec<Activity>, ApiError> {
    let pool = crate::server::auth::ssr::pool()?;
    sqlx::query_as::<_, Activity>(
        r#"SELECT a.id, a.user_id, name, rank_id, patch, layout, category, 
                    title_old, title_new, rank_old, rank_new, a.created_at
                FROM activity a
                INNER JOIN "user" u ON a.user_id = u.id
                LEFT JOIN rank r ON rank_id = r.id
                ORDER BY a.created_at DESC
                LIMIT 50 OFFSET $1;"#,
    )
    .bind(offset)
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError(
        "Database lookup failed".to_string(),
    )))
}
