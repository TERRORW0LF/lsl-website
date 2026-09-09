use leptos::prelude::*;
use server_fn::codec::GetUrl;
use types::{
    api::{ApiError, Permissions},
    internal::ssr::GetUser,
};

// TODO: Move rank and title updates into rating update trigger tied to rank table
#[server(RecalculateRankings, prefix="/api", endpoint="ranking/recalculate", input=GetUrl)]
pub async fn recalculate_ranks() -> Result<(), ApiError> {
    Ok(())
}

#[server(RecalculateRuns, prefix="/api", endpoint="runs/recalculate", input=GetUrl)]
pub async fn recalculate_runs(layout: String, category: String, map: String) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageRuns).ok_or(ApiError::Unauthorized)?;
    let pool = pool()?;

    let _ = sqlx::query(
        r#"UPDATE run
        SET points = GREATEST(3.0 - 2.0 * time::double precision 
            / (SELECT time 
               FROM run r
               WHERE run.section_id = r.section_id AND r.is_wr = true;)::double precision
            , 0.0)
        FROM section s
        WHERE section_id = s.id AND patch = '2.13' AND layout = $1 AND category = $2 AND map = $3;"#,
    )
    .bind(layout)
    .bind(category)
    .bind(map)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()));

    Ok(())
}

#[server(AddSection, prefix="/api", endpoint="section/add", input=GetUrl)]
pub async fn add_section(
    layout: String,
    category: String,
    map: String,
    code: String,
    submittable: bool,
) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::Administrator).ok_or(ApiError::Unauthorized)?;
    let pool = pool()?;

    let _ = sqlx::query(
        r#"INSERT INTO section (patch, layout, category, map, code, submittable)
        VALUES ('2.13', $1, $2, $3, $4, $5);"#,
    )
    .bind(layout)
    .bind(category)
    .bind(map)
    .bind(code)
    .bind(submittable)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database insert failed".into()));

    Ok(())
}

#[server(UpdateSetion, prefix="/api", endpoint="section/update", input=GetUrl)]
pub async fn update_section(id: i32, submittable: Option<bool>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::Administrator).ok_or(ApiError::Unauthorized)?;
    let pool = pool()?;

    let mut update = false;
    let mut query = sqlx::QueryBuilder::new(
        r#"UPDATE section
        SET"#,
    );
    if let Some(submit) = submittable {
        update = true;
        query.push(" submittable = ").push_bind(submit);
    }
    update.ok_or(ApiError::InvalidInput)?;
    let _ = query
        .push(" WHERE id = ")
        .push_bind(id)
        .build()
        .persistent(false)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()));

    Ok(())
}
