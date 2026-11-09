use bit_vec::BitVec;
use leptos::prelude::*;
use server_fn::codec::PostUrl;
use types::api::{ApiError, PartialRestrictedUser, Permissions, UserFilters, UserPermissions};

#[server(RecalculateRankings, prefix="/api", endpoint="ranking/recalculate", input=PostUrl)]
pub async fn recalculate_ranks(
    layout: Option<String>,
    category: Option<String>,
    redirect: Option<String>,
) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has_all(&[Permissions::ManageRuns, Permissions::ManageSections]).ok_or(ApiError::Unauthorized)?;
    let layout = layout.map(|v| if v.is_empty() { None } else { Some(v) }).flatten();
    let category = category.map(|v| if v.is_empty() { None } else { Some(v) }).flatten();
    if layout.is_some() && category.is_none() || (layout.is_none() && category.is_some()) {
        return Err(ApiError::InvalidInput);
    }
    let pool = pool()?;

    let _ = sqlx::query(
        r#"WITH ranking AS (SELECT distance, percent, scaled, rate, user_id 
            FROM (SELECT (COUNT(DISTINCT section_id)::double precision / (SELECT COUNT(id) 
                    FROM section 
                    WHERE patch = '2.13' AND ($1 IS NULL OR layout = $1) 
                        AND ($2 IS NULL OR category = $2) AND submittable = true))
                AS percent, AVG(points) AS distance, user_id
                FROM run
                JOIN section ON section.id = section_id
                WHERE patch = '2.13' AND ($1 IS NULL OR layout = $1) 
                    AND ($2 IS NULL OR category = $2) AND submittable = true AND verified = true
                GROUP BY user_id) base, 
            LATERAL (SELECT (percent ^ (1 / (1.3 + percent))) AS scaled) AS scale, 
            LATERAL (SELECT -10000 * scaled * (
			    (EXP((-distance + 1) ^ 
			    	(1 / (12 - 10.7 * (-EXP((-distance + 1) ^ 
		                (1 / (1.9 * (3 - 2 * distance) 
                            * (2 - scaled)))) 
    				+ EXP(1.0)) / (EXP(1.0) - 1)))) 
	    		- EXP(1.0)) / (EXP(1.0) - 1)) AS rate) AS res)
        MERGE INTO rank r
        USING ranking ra ON r.user_id = ra.user_id AND patch = '2.13'
            AND layout IS NOT DISTINCT FROM $1 AND category IS NOT DISTINCT FROM $2
        WHEN MATCHED THEN UPDATE SET 
            percentage = percent,
            points = distance,
		    rating = rate,
		    updated_at = now()
        WHEN NOT MATCHED BY TARGET THEN INSERT
            (user_id, patch, layout, category, title, rank, rating, percentage, points)
            VALUES (ra.user_id, '2.13', $1, $2, NULL, NULL, rate, percent, distance)
        WHEN NOT MATCHED BY SOURCE AND patch = '2.13' AND layout IS NOT DISTINCT FROM $1 
                AND category IS NOT DISTINCT FROM $2 THEN DELETE;"#,
    )
    .bind(&layout)
    .bind(&category)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    let _ = sqlx::query("CALL rank_update_rank('2.13', $1, $2, NULL);")
        .bind(layout)
        .bind(category)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    let _ = sqlx::query("CALL rank_update_rank('2.13', NULL, NULL, NULL);")
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    util::ssr::redirect(redirect);
    Ok(())
}

#[server(RecalculateRuns, prefix="/api", endpoint="runs/recalculate", input=PostUrl)]
pub async fn recalculate_runs(section_id: i32, redirect: Option<String>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageRuns).ok_or(ApiError::Unauthorized)?;
    (section_id > 1092).ok_or(ApiError::InvalidSection)?;
    let pool = pool()?;

    let mut tx = pool.begin().await.map_err(|_| ApiError::ServerError("Failed to start transaction.".into()))?;
    let _ = sqlx::query(
        r#"UPDATE run r1
        SET is_wr = (r1.verified = true AND r1.id = (SELECT id 
            FROM run r2 
            WHERE r2.section_id = $1 AND r2.verified
            ORDER BY r2.time ASC, r2.created_at ASC, r2.id ASC 
            LIMIT 1)),
        is_pb = (r1.verified = true AND r1.id = (SELECT id
            FROM run r2
            WHERE r2.section_id = $1 AND r2.user_id = r1.user_id AND r2.verified
            ORDER BY r2.time ASC, r2.created_at ASC, r2.id ASC
            LIMIT 1))
        WHERE section_id = $1;"#,
    )
    .bind(section_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    let _ = sqlx::query(
        r#"UPDATE run
        SET points = CASE
            WHEN is_pb
            THEN GREATEST(3.0 - 2.0 * time::double precision 
                / (SELECT time 
                    FROM run r
                    WHERE run.section_id = r.section_id AND r.is_wr = true)::double precision
                , 0.0)
            ELSE null
        END
        WHERE section_id = $1;"#,
    )
    .bind(section_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    tx.commit().await.map_err(|_| ApiError::ServerError("Failed to commit transaction.".into()))?;

    util::ssr::redirect(redirect);
    Ok(())
}

#[server(AddSection, prefix="/api", endpoint="section/add", input=PostUrl)]
pub async fn add_section(
    layout: String,
    category: String,
    map: String,
    code: String,
    submittable: Option<String>,
    redirect: Option<String>,
) -> Result<(), ApiError> {
    // TODO: add file uploads for map images
    use crate::auth::ssr::{auth, pool};
    use regex::regex;

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageSections).ok_or(ApiError::Unauthorized)?;
    (regex!("^[[:alnum:]:\\-() ]{4,32}$").is_match(&map) && regex!("^[A-Z0-9]{4}$").is_match(&code))
        .ok_or(ApiError::InvalidInput)?;
    let pool = pool()?;

    let submittable = submittable.is_some_and(|v| !v.is_empty());

    let num = sqlx::query(
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

    if num.rows_affected() != 0 {
        util::ssr::redirect(redirect);
        Ok(())
    } else {
        Err(ApiError::AlreadyExists)
    }
}

#[server(UpdateSection, prefix="/api", endpoint="section/update", input=PostUrl)]
pub async fn update_section(
    id: i32,
    submittable: Option<String>,
    name: Option<String>,
    code: Option<String>,
    redirect: Option<String>,
) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};
    use regex::regex;

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageSections).ok_or(ApiError::Unauthorized)?;
    (id > 1092).ok_or(ApiError::InvalidSection)?;
    let pool = pool()?;

    let mut update = false;
    let mut query = sqlx::QueryBuilder::new(
        r#"UPDATE section
        SET"#,
    );
    let mut set = query.separated(", ");
    if let Some(submit) = submittable
        && !submit.is_empty()
    {
        update = true;
        match submit.as_str() {
            "true" => {
                set.push(" submittable = true");
            }
            "false" => {
                set.push(" submittable = false");
            }
            _ => Err(ApiError::InvalidInput)?,
        }
    }
    // TODO: Tie image to map in db
    if let Some(name) = name
        && let Some(code) = code
        && !name.is_empty()
        && !code.is_empty()
    {
        (regex!("^[[:alnum:]:\\-() ]{4,32}$").is_match(&name) && regex!("^[A-Z0-9]{4}$").is_match(&code))
            .ok_or(ApiError::InvalidInput)?;
        update = true;
        set.push(" map = ").push_bind_unseparated(name).push(" code = ").push_bind_unseparated(code);
    }
    update.ok_or(ApiError::InvalidInput)?;
    let num = query
        .push(" WHERE id = ")
        .push_bind(id)
        .push(";")
        .build()
        .persistent(false)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database update failed".into()))?;

    if num.rows_affected() != 0 {
        util::ssr::redirect(redirect);
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

#[server(DeleteSection, prefix="/api", endpoint="section/delete", input=PostUrl)]
pub async fn delete_section(id: i32, redirect: Option<String>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let auth = auth()?;
    let pool = pool()?;

    let u = auth.current_user.ok_or(ApiError::Unauthenticated)?;
    u.has(&Permissions::ManageSections).ok_or(ApiError::Unauthorized)?;
    (id > 1092).ok_or(ApiError::InvalidSection)?;

    let num = sqlx::query(
        r#"DELETE FROM section
        WHERE id = $1 AND patch = '2.13';"#,
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::ServerError("Database update failed".into()))?;
    if num.rows_affected() != 0 {
        util::ssr::redirect(redirect);
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

#[server(UpdateRun, prefix="/api", endpoint="run/update", input=PostUrl)]
pub async fn update_run(
    id: i32,
    verified: Option<String>,
    section_id: Option<i32>,
    redirect: Option<String>,
) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    let pool = pool()?;

    let mut update = false;
    let mut query = sqlx::QueryBuilder::new(
        r#"UPDATE run
        SET"#,
    );
    let mut set = query.separated(", ");
    if verified.is_some_and(|v| !v.is_empty()) {
        user.has(&Permissions::Verify).ok_or(ApiError::Unauthorized)?;
        update = true;
        set.push(" verified = true");
    }
    if let Some(section_id) = section_id {
        user.has(&Permissions::ManageRuns).ok_or(ApiError::Unauthorized)?;
        (section_id > 1092).ok_or(ApiError::InvalidSection)?;
        update = true;
        set.push(" section_id = ").push_bind_unseparated(section_id);
    }
    update.ok_or(ApiError::InvalidInput)?;
    let num = query
        .push(" WHERE id = ")
        .push_bind(id)
        .push(" AND section_id > 1092;")
        .build()
        .persistent(false)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database udpate failed".into()))?;

    if num.rows_affected() != 0 {
        util::ssr::redirect(redirect);
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

#[server(GetUsers, prefix="/api", endpoint="users/get", input=PostUrl)]
pub async fn get_users(filter: UserFilters, offset: i32) -> Result<Vec<PartialRestrictedUser>, ApiError> {
    use crate::auth::ssr::{auth, pool};
    use sqlx::{Postgres, QueryBuilder};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageUsers).ok_or(ApiError::Unauthorized)?;
    let mut perms = BitVec::from_elem(64, false);
    for perm in filter.permission {
        perms.set(perm as usize, true);
    }
    let pool = pool()?;

    let mut query = QueryBuilder::<Postgres>::new(
        r#"SELECT id, name, bio, pfp, permission, created_at
        FROM "user"
        WHERE 1 = 1 AND permission = permission | "#,
    );
    query.push_bind(perms);

    if let Some(id) = filter.id
        && id != 0
    {
        query.push(" AND id = ").push_bind(id);
    }
    if let Some(name) = filter.name
        && !name.is_empty()
    {
        query.push(" AND name = ").push_bind(name);
    }
    let asc = match filter.ascending {
        true => "ASC",
        false => "DESC",
    };
    query
        .push(" ORDER BY ")
        .push(match filter.sort.as_str() {
            "created_at" => format!("created_at {asc}"),
            _ => format!("name {asc}"),
        })
        .push(" LIMIT 50 OFFSET ")
        .push_bind(offset)
        .push(";");

    query
        .build_query_as::<PartialRestrictedUser>()
        .fetch_all(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database lookup failed.".into()))
}

#[server(UpdateUser, prefix="/api", endpoint="user/update", input=PostUrl)]
pub async fn update_user(id: i64, permissions: Vec<Permissions>, redirect: Option<String>) -> Result<(), ApiError> {
    use crate::auth::ssr::{auth, pool};

    let user = auth()?.current_user.ok_or(ApiError::Unauthenticated)?;
    user.has(&Permissions::ManageUsers).ok_or(ApiError::Unauthorized)?;
    let max = user.permission.iter().enumerate().filter_map(|(i, b)| b.then(|| i as i32)).last().unwrap_or(-1);
    let mut perms = BitVec::from_elem(64, false);
    for perm in permissions {
        if user.has(&perm) && (perm as i32) < max {
            perms.set(perm as usize, true);
        }
    }
    let pool = pool()?;

    let num = sqlx::query(r#"UPDATE "user" SET permission = $1 WHERE id = $2;"#)
        .bind(perms)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::ServerError("Database udpate failed".into()))?;

    if num.rows_affected() != 0 {
        util::ssr::redirect(redirect);
        auth()?.cache_clear_user(user.id);
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}
