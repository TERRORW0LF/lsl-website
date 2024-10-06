use chrono::{DateTime, Utc};
use leptos::{server, ServerFnError};
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
    pub verified: bool,
    pub is_pb: bool,
    pub is_wr: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow, sqlx::Type))]
pub struct PartialRun {
    pub id: i32,
    pub section_id: i32,
    pub user_id: i64,
    pub username: String,
    pub time: Decimal,
    pub proof: String,
    pub verified: bool,
    pub is_pb: bool,
    pub is_wr: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MapRuns {
    /// Section id
    pub id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub runs: Vec<PartialRun>,
}

#[server(GetRunsId, "/api", "Url", "runs/id")]
pub async fn get_runs_id(id: i32, pb_only: bool) -> Result<MapRuns, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    sqlx::query_as::<_, MapRuns>(
        &(r#"SELECT s.id, s.patch, s.layout, s.category, s.map,
                COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, u.id,
                        u."name", r.time, r.proof, r.verified, r.is_pb, r.is_wr, r.created_at)
                    ORDER BY r.time ASC)"#
            .to_string()
            + if pb_only {
                "FILTER(WHERE r.is_pb = TRUE)"
            } else {
                ""
            }
            + r#", 
                '{NULL}'), '{}')
                AS "runs: Vec<PartialRun>"
            FROM section s
            INNER JOIN run r ON section_id = s.id
            INNER JOIN "user" u ON user_id = u.id
            WHERE s.id = $1
            GROUP BY s.id, patch, layout, category, map"#),
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .or(Err(ServerFnError::ServerError(
        "Failed to look up runs".into(),
    )))
}

#[server(GetRunsCategory, "/api", "Url", "runs/category")]
pub async fn get_runs_category(
    patch: String,
    layout: String,
    category: String,
    pb_only: bool,
) -> Result<Vec<MapRuns>, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    sqlx::query_as::<_, MapRuns>(
        &(r#"SELECT s.id, patch, layout, category, map,
                COALESCE(NULLIF(ARRAY_AGG((r.id, r.section_id, r.user_id,
                        u."name", r.time, r.proof, r.verified, r.is_pb, r.is_wr, r.created_at)
                    ORDER BY r.time ASC)"#
            .to_string()
            + if pb_only {
                "FILTER(WHERE r.is_pb = TRUE)"
            } else {
                ""
            }
            + r#",
                '{NULL}'), '{}')
                AS "runs: Vec<PartialRun>"
            FROM section s
            LEFT JOIN run r ON section_id = s.id
            LEFT JOIN "user" u ON user_id = u.id
            WHERE patch = $1 AND layout = $2 AND category = $3
            GROUP BY s.id, patch, layout, category, map"#),
    )
    .bind(patch)
    .bind(layout)
    .bind(category)
    .fetch_all(&pool)
    .await
    .or_else(|e| Err(ServerFnError::ServerError(e.to_string())))
}

#[server(GetRunsLatest, "/api", "Url", "runs/latest")]
pub async fn get_runs_latest(offset: i32) -> Result<Vec<Run>, ServerFnError> {
    let pool = crate::auth::ssr::pool()?;
    sqlx::query_as::<_, Run>(
        r#"SELECT run.id, run.created_at, section_id, patch,
            layout, category, map, user_id, "name", time, proof, verified, is_pb, is_wr
        FROM run
        INNER JOIN section ON section_id = section.id
        INNER JOIN "user" ON user_id = u.id
        LIMIT 50 OFFSET $1"#,
    )
    .bind(offset)
    .fetch_all(&pool)
    .await
    .or(Err(ServerFnError::ServerError(
        "Failed to look up runs".into(),
    )))
}
