use leptos::prelude::*;
use server_fn::codec::GetUrl;
use types::api::{ApiError, Permissions};

// TODO: Move rank and title updates into rating update trigger tied to rank table
#[server(RecalculateRankings, prefix="/api", endpoint="ranking/recalculate", input=GetUrl)]
pub async fn recalculate_ranks(layout: Option<String>, category: Option<String>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};
    use types::internal::ssr::GetUser;

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageRuns).ok_or(ApiError::Unauthorized)?;
    if layout.is_some() && category.is_none() || (layout.is_none() && category.is_some()) {
        return Err(ApiError::InvalidInput);
    }
    let pool = pool()?;

    let _ = sqlx::query(
        r#"WITH ran AS (SELECT (percentage ^ (1 / (1.3 + percentage))) AS p, user_id AS u 
				FROM rank
				WHERE patch = '2.13' AND layout IS NOT DISTINCT FROM $1 
					AND category IS NOT DISTINCT FROM $2),
		po AS (SELECT AVG(points) as p, user_id as u 
			FROM run r 
			JOIN section s ON section_id = s.id
			WHERE s.patch = '2.13' AND layout = $1
				AND category = $2 AND submittable = true
			GROUP BY user_id)
		UPDATE rank r
		SET points = (SELECT p FROM po WHERE u = r.user_id),
		rating = (-10000 * (SELECT p FROM ran WHERE u = r.user_id) * (
			(EXP((-(SELECT p FROM po WHERE u = r.user_id) + 1) ^ 
				(1 / (12 - 10.7 * (-EXP((-(SELECT p FROM po WHERE u = r.user_id) + 1) ^ 
		            (1 / (1.9 * (3 - 2 * (SELECT p FROM po WHERE u = r.user_id)) 
                        * (2 - (SELECT p FROM ran WHERE u = r.user_id))))) 
				+ EXP(1.0)) / (EXP(1.0) - 1)))) 
			- EXP(1.0)) / (EXP(1.0) - 1))),
		updated_at = NEW.created_at
		WHERE patch = '2.13' AND layout IS NOT DINSTINCT FROM $1 AND category IS NOT DISTINCT FROM $2;"#,
    )
    .bind(layout)
    .bind(category)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    Ok(())
}

#[server(RecalculateRuns, prefix="/api", endpoint="runs/recalculate", input=GetUrl)]
pub async fn recalculate_runs(layout: String, category: String, map: String) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};
    use types::internal::ssr::GetUser;

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
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

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
    use types::internal::ssr::GetUser;

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
    .map_err(|_| ApiError::ServerError("Database insert failed".into()))?;

    Ok(())
}

#[server(UpdateSetion, prefix="/api", endpoint="section/update", input=GetUrl)]
pub async fn update_section(id: i32, submittable: Option<bool>, name: Option<String>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};
    use types::internal::ssr::GetUser;

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::Administrator).ok_or(ApiError::Unauthorized)?;
    let pool = pool()?;

    let mut update = false;
    let mut query = sqlx::QueryBuilder::new(
        r#"UPDATE section
        SET"#,
    );
    let mut set = query.separated(", ");
    if let Some(submit) = submittable {
        update = true;
        set.push(" submittable = ").push_bind_unseparated(submit);
    }
    if let Some(name) = name {
        update = true;
        set.push(" map = ").push_bind_unseparated(name);
    }
    update.ok_or(ApiError::InvalidInput)?;
    let _ = query
        .push(" WHERE id = ")
        .push_bind(id)
        .build()
        .persistent(false)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    Ok(())
}
