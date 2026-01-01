use http::{HeaderValue, header::CACHE_CONTROL};
use leptos::prelude::{expect_context, server, server_fn::codec::GetUrl};
use rust_decimal::Decimal;
use types::api::*;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
struct RunFull {
    id: i32,
    created_at: chrono::DateTime<chrono::Local>,
    section_id: i32,
    user_id: i64,
    time: Decimal,
    proof: String,
    yt_id: Option<String>,
    verified: bool,
}

#[server(ExportRuns, prefix="/api", endpoint="runs/export", input=GetUrl)]
pub async fn export_runs(patch: String) -> Result<(), ApiError> {
    let pool = crate::auth::ssr::pool()?;
    let runs = sqlx::query_as::<_, RunFull>(
        r#"SELECT run.id, run.created_at, section_id, user_id, time, proof, yt_id, verified
        FROM run
        JOIN section on run.section_id = section.id
        WHERE patch = $1
        ORDER BY created_at ASC;"#,
    )
    .bind(patch.clone())
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError("Couldn't fetch runs".into())))?;

    let file = std::fs::File::create(format!("{patch}.json")).unwrap();
    let writer = std::io::BufWriter::new(file);
    leptos::serde_json::to_writer_pretty(writer, &runs).expect("Couldn't write to file");
    Ok(())
}

#[server(ImportRuns, prefix="/api", endpoint="runs/import", input=GetUrl)]
pub async fn import_runs(patch: String) -> Result<(), ApiError> {
    let pool = crate::auth::ssr::pool()?;
    let file = std::fs::File::open(format!("{patch}.json")).expect("Can't find file");
    let reader = std::io::BufReader::new(file);
    let runs: Vec<RunFull> = leptos::serde_json::from_reader(reader).expect("Can't parse file");
    let _ = sqlx::query(
        r#"DELETE FROM run
            USING section
            WHERE run.section_id = section.id AND patch = $1;"#,
    )
    .bind(patch.clone())
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database delete failed".into()))?;

    let _ = sqlx::query(
        r#"DELETE FROM rank
            WHERE patch = $1;"#,
    )
    .bind(patch)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database delete failed".into()))?;

    let _ = sqlx::query(
        r#"SELECT setval(pg_get_serial_sequence('rank', 'id'), COALESCE(MAX(id) + 1, 1), FALSE) 
        FROM rank;"#,
    )
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Sequence reset failed".into()))?;

    for run in runs {
        let _ = sqlx::query(
            r#"INSERT INTO run (id, section_id, user_id, time, proof, yt_id, verified, created_at)
            OVERRIDING SYSTEM VALUE
                                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8);"#,
        )
        .bind(run.id)
        .bind(run.section_id)
        .bind(run.user_id)
        .bind(run.time)
        .bind(run.proof)
        .bind(run.yt_id)
        .bind(true)
        .bind(run.created_at)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database insert failed".into()))?;
    }
    Ok(())
}

#[server(GetRunsId, prefix="/api", endpoint="runs/id", input=GetUrl)]
pub async fn get_runs_id(id: i32) -> Result<SectionRuns, ApiError> {
    let pool = crate::auth::ssr::pool()?;
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
pub async fn get_runs_category(patch: String, layout: String, category: String) -> Result<Vec<SectionRuns>, ApiError> {
    let pool = crate::auth::ssr::pool()?;
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

#[server(GetRuns, prefix="/api", endpoint="runs/user", input=GetUrl)]
pub async fn get_runs(filter: RunFilters, offset: i32) -> Result<Vec<Run>, ApiError> {
    use sqlx::{Postgres, QueryBuilder};

    let pool = crate::auth::ssr::pool()?;
    let mut query = QueryBuilder::<Postgres>::new(
        r#"SELECT run.id, run.created_at, section_id, patch, layout, 
            category, map, user_id, "name", time, proof, yt_id, verified, is_pb, is_wr
        FROM run
        INNER JOIN section s ON section_id = s.id
        INNER JOIN "user" u ON user_id = u.id 
        WHERE 1 = 1"#,
    );
    if let Some(user) = filter.user {
        query.push(" AND user_id = ").push_bind(user);
    }
    if let Some(before) = filter.before {
        query.push(" AND run.created_at <= ").push_bind(before);
    }
    if let Some(after) = filter.after {
        query.push(" AND run.created_at >= ").push_bind(after);
    }
    if let Some(patch) = filter.patch {
        query.push(" AND patch = ").push_bind(patch);
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
    let asc = match filter.ascending {
        true => "ASC",
        false => "DESC",
    };
    query
        .push(" ORDER BY ")
        .push(match filter.sort.as_str() {
            "section" => format!("patch {asc}, layout {asc}, category {asc}, map {asc}"),
            "time" => format!("time {asc}"),
            _ => format!("run.created_at {asc}"),
        })
        .push(" LIMIT 50 OFFSET ")
        .push_bind(offset)
        .push(";");
    query
        .build_query_as::<Run>()
        .fetch_all(&pool)
        .await
        .or(Err(ApiError::ServerError("Database lookup failed".to_string())))
}

#[server(GetMaps, prefix="/api", endpoint="maps", input=GetUrl)]
pub async fn get_maps() -> Result<Vec<Map>, ApiError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let maps = sqlx::query_as::<_, Map>(
        r#"SELECT map, code
        FROM section
        WHERE patch='2.13' AND layout='1' AND category='Standard';"#,
    )
    .fetch_all(&pool)
    .await
    .or(Err(ApiError::ServerError("Database lookup failed".to_string())))?;

    res_opts.append_header(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
    Ok(maps)
}

#[server(GetUser, prefix="/api", endpoint="user/get", input=GetUrl)]
pub async fn get_user(id: i64) -> Result<User, ApiError> {
    use crate::auth::ssr::*;

    let pool = pool()?;
    User::get(id, &pool).await.ok_or(ApiError::NotFound)
}

#[server(GetRankings, prefix="/api", endpoint="ranking", input=GetUrl)]
pub async fn get_rankings(
    patch: String,
    layout: Option<String>,
    category: Option<String>,
) -> Result<Vec<Ranking>, ApiError> {
    let pool = crate::auth::ssr::pool()?;
    let res_opts = expect_context::<leptos_axum::ResponseOptions>();
    let rankings = sqlx::query_as::<_, Ranking>(
        r#"SELECT r.id, r.patch, r.layout, r.category, r.user_id, 
            u.name, r.title, r.rank, r.rating, r.created_at, r.updated_at, r.percentage, r.points
        FROM rank r
        JOIN "user" u ON user_id = u.id
        WHERE r.patch = $1 AND r.layout IS NOT DISTINCT FROM $2 AND r.category IS NOT DISTINCT FROM $3
        ORDER BY r.rating DESC, r.updated_at ASC;"#,
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
    let pool = crate::auth::ssr::pool()?;

    sqlx::query_as::<_, Ranking>(
        r#"SELECT r.id, r.patch, r.layout, r.category, r.user_id, 
            u.name, r.title, r.rank, r.rating, r.created_at, r.updated_at, r.percentage, r.points
        FROM rank r
        JOIN "user" u ON user_id = u.id
        WHERE user_id = $1;"#,
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database lookup failed".into()))
}

#[server(GetRandUser, prefix="/api", endpoint="user/get/random", input=GetUrl)]
pub async fn get_rand_user() -> Result<User, ApiError> {
    use crate::auth::ssr::*;

    let pool = pool()?;
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
pub async fn get_activity(filter: ActivityFilters, offset: i32) -> Result<Vec<Activity>, ApiError> {
    use sqlx::{Postgres, QueryBuilder};

    let pool = crate::auth::ssr::pool()?;
    let mut query = QueryBuilder::<Postgres>::new(
        r#"SELECT a.id, a.user_id, name, rank_id, patch, layout, category, 
                    title_old, title_new, rank_old, rank_new, a.created_at
                FROM activity a
                INNER JOIN "user" u ON a.user_id = u.id
                LEFT JOIN rank r ON rank_id = r.id
                WHERE 1 = 1"#,
    );
    if let Some(event) = filter.event {
        match event.as_str() {
            "join" => query.push(" AND rank_id IS NULL"),
            "rank" => query.push(" AND rank_new IS NOT NULL"),
            "title" => query.push(" AND title_new IS NOT NULL"),
            _ => &mut query,
        };
    }
    if let Some(user) = filter.user {
        query.push(" AND a.user_id = ").push_bind(user);
    }
    if let Some(patch) = filter.patch {
        query.push(" AND patch = ").push_bind(patch);
    }
    if let Some(layout) = filter.layout {
        query.push(" AND layout = ").push_bind(layout);
    }
    if let Some(category) = filter.category {
        query.push(" AND category = ").push_bind(category);
    }
    if let Some(before) = filter.before {
        query.push(" AND a.created_at <= ").push_bind(before);
    }
    if let Some(after) = filter.after {
        query.push(" AND a.created_at >= ").push_bind(after);
    }
    let asc = match filter.ascending {
        true => " ASC",
        false => " DESC",
    };
    query
        .push(" ORDER BY ")
        .push(match filter.sort.as_str() {
            "section" => format!("patch {asc}, layout {asc}, category {asc}"),
            _ => format!("a.created_at {asc}"),
        })
        .push(" LIMIT 50 OFFSET ")
        .push_bind(offset)
        .push(";");

    query
        .build_query_as::<Activity>()
        .fetch_all(&pool)
        .await
        .or(Err(ApiError::ServerError("Database lookup failed".to_string())))
}
