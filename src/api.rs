use chrono::{DateTime, Local};
use http::{header::CACHE_CONTROL, HeaderValue};
use leptos::prelude::{expect_context, server, server_fn::codec::GetUrl, ServerFnError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Map(String);

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[server(GetRunsId, prefix="/api", endpoint="runs/id", input=GetUrl)]
pub async fn get_runs_id(id: i32) -> Result<MapRuns, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let res = sqlx::query_as::<_, MapRuns>(
        r#"SELECT s.id, s.patch, s.layout, s.category, s.map,
                COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, u.id, u."name", r.time,
                        r.proof, r.yt_id, r.verified, r.is_pb, r.is_wr, r.created_at)
                    ORDER BY r.created_at ASC)
                    FILTER(WHERE r.id IS NOT NULL), '{NULL}'), '{}') AS runs
            FROM section s
            INNER JOIN run r ON section_id = s.id
            INNER JOIN "user" u ON user_id = u.id
            WHERE s.id = $1
            GROUP BY s.id, patch, layout, category, map"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(runs) => {
            res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
            Ok(runs)
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Database lookup failed".to_string(),
        )),
    }
}

#[server(GetRunsCategory, prefix="/api", endpoint="runs/category", input=GetUrl)]
pub async fn get_runs_category(
    patch: String,
    layout: String,
    category: String,
) -> Result<Vec<MapRuns>, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let res = sqlx::query_as::<_, MapRuns>(
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
    .await;

    match res {
        Ok(runs) => {
            res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=900"));
            Ok(runs)
        }
        Err(e) => {
            tracing::warn!("{}", e.to_string());
            Err(ServerFnError::ServerError(
                "Database lookup failed".to_string(),
            ))
        }
    }
}

#[server(GetRunsLatest, prefix="/api", endpoint="runs/latest", input=GetUrl)]
pub async fn get_runs_latest(offset: i32) -> Result<Vec<Run>, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let res = sqlx::query_as::<_, Run>(
        r#"SELECT run.id, run.created_at, section_id, patch,
            layout, category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
        FROM run
        INNER JOIN section ON section_id = section.id
        INNER JOIN "user" ON user_id = u.id
        ORDER BY run.created_at DESC
        LIMIT 50 OFFSET $1"#,
    )
    .bind(offset)
    .fetch_all(&pool)
    .await;

    match res {
        Ok(runs) => {
            res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=300"));
            Ok(runs)
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Database lookup failed".to_string(),
        )),
    }
}

#[server(GetMaps, prefix="/api", endpoint="maps", input=GetUrl)]
pub async fn get_maps(
    patch: String,
    layout: String,
    category: String,
) -> Result<Vec<Map>, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let res = sqlx::query_as::<_, Map>(
        r#"SELECT map
        FROM section
        WHERE patch=$1 AND layout=$2 AND category=$3"#,
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await;

    match res {
        Ok(maps) => {
            res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
            Ok(maps)
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Database lookup failed".to_string(),
        )),
    }
}
