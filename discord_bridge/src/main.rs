#![feature(future_join)]
use std::future::join;

use chrono::{DateTime, Local, TimeDelta};
use log::debug;
use reqwest::Client;
use serde_json::json;
use sqlx::postgres::{PgConnectOptions, PgListener};
use sqlx::prelude::FromRow;
use sqlx::{query, query_as, PgPool};
use types::{api::*, internal::ssr::AuthRes};
use urlencoding::encode;

#[derive(Clone, FromRow)]
struct Discord {
    id: i32,
    user_id: i64,
    access: String,
    refresh: String,
    expires_at: DateTime<Local>,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_env().expect("couldn't initialize logging");

    let submit_client = Client::new();
    let activity_client = submit_client.clone();
    let discord_client = submit_client.clone();

    let connect_opts = PgConnectOptions::new()
        .database(&std::env::var("PG_DB").unwrap())
        .username(&std::env::var("PG_USER").unwrap())
        .password(&std::env::var("PG_PASS").unwrap())
        .host(&std::env::var("PG_HOST").unwrap())
        .port(std::env::var("PG_PORT").unwrap().parse::<u16>().unwrap());

    let submit_pool = PgPool::connect_with(connect_opts).await.unwrap();
    let activity_pool = submit_pool.clone();
    let discord_pool = submit_pool.clone();
    let submit = tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&submit_pool).await.unwrap();
        listener.listen("submit").await.unwrap();
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let run = query_as::<_, Run>(
                        r#"SELECT r.id, r.user_id, u.name, r.section_id, s.patch, s.layout, s.category, 
                            s.map, r.time, r.proof, r.verified, r.yt_id, r.is_pb, r.is_wr, r.created_at
                        FROM run r
                        INNER JOIN "user" u ON r.user_id = u.id
                        INNER JOIN section s ON r.section_id = s.id
                        WHERE r.id = $1::integer;"#,
                    )
                    .bind(notification.payload())
                    .fetch_one(&submit_pool)
                    .await;

                    match run {
                        Ok(r) => {
                            if r.is_wr {
                                let old = query_as::<_, PartialRun>(
                                    r#"SELECT r.id, r.section_id, r.user_id, u.name, r.time, r.proof, 
                                        r.verified, r.yt_id, r.is_pb, r.is_wr, r.created_at
                                    FROM run r
                                    INNER JOIN "user" u ON r.user_id = u.id
                                    WHERE section_id = $1
                                    ORDER BY time ASC, created_at ASC
                                    OFFSET 1
                                    LIMIT 1;"#,
                                )
                                .bind(r.section_id)
                                .fetch_optional(&submit_pool)
                                .await
                                .ok()
                                .flatten();
                                send_wr(&r, &old, &submit_client).await;
                            } else if r.is_pb {
                                let old = query_as::<_, PartialRun>(
                                    r#"SELECT r.id, r.section_id, r.user_id, u.name, r.time, r.proof, 
                                        r.verified, r.yt_id, r.is_pb, r.is_wr, r.created_at
                                    FROM run r
                                    INNER JOIN "user" u ON r.user_id = u.id
                                    WHERE section_id = $1 AND user_id = $2
                                    ORDER BY time ASC, created_at ASC
                                    OFFSET 1
                                    LIMIT 1;"#,
                                )
                                .bind(r.section_id)
                                .bind(r.user_id)
                                .fetch_optional(&submit_pool)
                                .await
                                .ok()
                                .flatten();
                                send_pb(&r, &old, &submit_client).await;
                            }
                        }
                        Err(e) => debug!("{e:?}"),
                    }
                }
                Err(e) => debug!("{e:?}"),
            };
        }
    });
    let activity = tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&activity_pool).await.unwrap();
        listener.listen("activity").await.unwrap();
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let activity = query_as::<_, Activity>(
                        r#"SELECT a.id, a.user_id, u.name, a.rank_id, 
                            r.patch, r.layout, r.category, a.title_old, 
                            a.title_new, a.rank_old, a.rank_new, a.created_at
                        FROM activity a
                        INNER JOIN "user" u ON a.user_id = u.id
                        INNER JOIN rank r ON a.rank_id = r.id
                        WHERE a.id = $1::integer;"#,
                    )
                    .bind(notification.payload())
                    .fetch_one(&activity_pool)
                    .await;

                    match activity {
                        Ok(a) => {
                            if a.title_old.is_some() && a.title_new.is_some() {
                                send_title(&a, &activity_client).await;
                            } else if a.rank_old.is_some() && a.rank_new.is_some() {
                                send_rank(&a, &activity_client).await;
                            } else {
                                send_join(&a, &activity_client).await;
                            }
                            if a.title_new.is_some() && a.layout.is_none() && a.category.is_none() {
                                let discord = query_as::<_, Discord>(
                                    r#"SELECT id, user_id, access, refresh, expires_at
                                    FROM discord
                                    WHERE user_id = $1;"#
                                )
                                .bind(a.user_id)
                                .fetch_all(&activity_pool)
                                .await;

                                match discord {
                                    Ok(d) => update_title(&a, &d, &activity_client, &activity_pool).await,
                                    Err(_) => {},
                                }
                            }
                        }
                        Err(e) => debug!("{e:?}"),
                    }
                }
                Err(e) => debug!("{e:?}"),
            };
        }
    });
    let discord = tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&discord_pool).await.unwrap();
        listener.listen("discord").await.unwrap();
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let discord = query_as::<_, Discord>(
                        r#"SELECT id, user_id, access, refresh, expires_at
                        FROM discord
                        WHERE id = $1::integer;"#
                    )
                    .bind(notification.payload())
                    .fetch_one(&discord_pool)
                    .await;

                    match discord {
                        Ok(d) => {
                            let ranks = query_as::<_, Ranking>(
                                r#"SELECT r.id, r.patch, r.layout, r.category, r.user_id, 
                                u.name, r.title, r.rank, r.rating, r.created_at, r.updated_at
                                FROM rank r
                                INNER JOIN "user" u ON r.user_id = u.id
                                WHERE r.user_id = $1 AND r.layout IS NULL AND r.category IS NULL;"#
                            )
                            .bind(d.user_id)
                            .fetch_all(&discord_pool)
                            .await;
                            match ranks {
                                Ok(r) => set_title(&r, &d, &discord_client, &discord_pool).await,
                                Err(_) => {},
                            }
                        }
                        Err(e) => debug!("{e:?}"),
                    }
                }
                Err(e) => debug!("{e:?}"),
            }
        }
    });

    let _ = join!(submit, activity, discord).await;
}

async fn send_pb(new: &Run, old: &Option<PartialRun>, client: &Client) {
    let _ = client.post(std::env::var("PB_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": 16764928,
            "title": format!("New Personal Best by {}", new.username),
            "url": format!("https://youtube.com/watch?v={}", new.yt_id.as_ref().unwrap()),
            "thumbnail": { "url": format!("https://lucio.surf/cdn/maps/{}.jpg", encode(&new.map))},
            "description": format!("Patch: *{}*\nLayout: *{}*\nCategory: *{}*\nMap: *{}*", 
                new.patch, new.layout, new.category, new.map),
            "fields": [{
                "name": "New PB",
                "value": format!("Time: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                    new.time.to_string(), new.yt_id.as_ref().unwrap(), new.created_at.timestamp()),
                "inline": true
            },
            {
                "name": "Old PB",
                "value": match old {
                    Some(o) => format!("Time: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                        o.time.to_string(), o.yt_id.as_ref().unwrap(), o.created_at.timestamp()),
                    None => "User: *none*\nTime: *none*\nProof: *none*\nDate: *none*".into(),
                },
                "inline": true
            },
            {
                "name": "Comparison",
                "value": match old {
                    Some(o) => {
                        let diff = new.created_at - o.created_at;
                        format!("Time save: *{}*\nAchieved after: *{} weeks, {} days, {} hours, {} mins, {} secs*", 
                            (o.time - new.time).to_string(), diff.num_weeks(), diff.num_days() % 7, 
                            diff.num_hours() % 24, diff.num_minutes() % 60, diff.num_seconds() % 60
                        )},
                    None => "Time save: *undefined*\nAchieved after: *undefined*".into(),
                }
            }],
            "footer": { "text": format!("ID: {}", new.id) }
        }] 
    })).send().await;
    let _ = client.post(std::env::var("PB_WEBHOOK").unwrap()).json(&json!({
        "content": format!("https://youtube.com/watch?v={}", new.yt_id.as_ref().unwrap())
    })).send().await;
}

async fn send_wr(new: &Run, old: &Option<PartialRun>, client: &Client) {
    let _ = client.post(std::env::var("WR_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": 7798548,
            "title": format!("New World Record by {}", new.username),
            "url": format!("https://youtube.com/watch?v={}", new.yt_id.as_ref().unwrap()),
            "thumbnail": { "url": format!("https://lucio.surf/cdn/maps/{}.jpg", encode(&new.map))},
            "description": format!("Patch: *{}*\nLayout: *{}*\nCategory: *{}*\nMap: *{}*", 
                new.patch, new.layout, new.category, new.map),
            "fields": [{
                "name": "New Record",
                "value": format!("User: *{}*\nTime: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                    new.username, new.time.to_string(), new.yt_id.as_ref().unwrap(), new.created_at.timestamp()),
                "inline": true
            },
            {
                "name": "Old Record",
                "value": match old {
                    Some(o) => format!("User: *{}*\nTime: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                        o.name, o.time.to_string(), o.yt_id.as_ref().unwrap(), o.created_at.timestamp()),
                    None => "User: *none*\nTime: *none*\nProof: *none*\nDate: *none*".into(),
                },
                "inline": true
            },
            {
                "name": "Comparison",
                "value": match old {
                    Some(o) => {
                        let diff = new.created_at - o.created_at;
                        format!("Time save: *{}*\nAchieved after: *{} weeks, {} days, {} hours, {} mins, {} secs*", 
                            (o.time - new.time).to_string(), diff.num_weeks(), diff.num_days() % 7, 
                            diff.num_hours() % 24, diff.num_minutes() % 60, diff.num_seconds() % 60
                        )},
                    None => "Time save: *undefined*\nAchieved after: *undefined*".into(),
                }
            }],
            "footer": { "text": format!("ID: {}", new.id) }
        }] 
    })).send().await;
    let _ = client.post(std::env::var("WR_WEBHOOK").unwrap()).json(&json!({
        "content": format!("https://youtube.com/watch?v={}", new.yt_id.as_ref().unwrap())
    })).send().await;
}

async fn send_title(activity: &Activity, client: &Client) {
    let new = activity.title_new.as_ref().unwrap();
    let old = activity.title_old.as_ref().unwrap();
    let _ = client.post(std::env::var("ACTIVITY_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": if new > old { 7798548 } else { 12064000 },
            "title": "Title update",
            "description": match (activity.layout.as_ref(), activity.category.as_ref()) {
                (Some(l), Some(c)) => format!("User: *{}*\nCombo: *Layout {} - {}*", 
                    activity.username, l, c),
                _ => format!("User: *{}*\nCombo: *Combined*", activity.username),
            },
            "fields": [{
                "name": old.to_string(),
                "value": "",
                "inline": true
            },
            {
                "name": new.to_string(),
                "value": "",
                "inline": true
            }]
        }]
    })).send().await;
}

async fn send_rank(_activity: &Activity, _client: &Client) {}

async fn send_join(activity: &Activity, client: &Client) {
    let _ = client.post(std::env::var("ACTIVITY_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": 1342207,
            "title": format!("{} joined the leaderboards!", activity.username)
        }]
    })).send().await;
}

async fn update_title(activity: &Activity, auth: &Vec<Discord>, client: &Client, pool: &PgPool) {
    for tokens in auth {
        let token = get_access_token(tokens, client, pool).await;
        if token.is_err() {
            return;
        }
        let _ = client
            .put(format!("https://discord.com/api/v10/users/@me/applications/{}/role-connection", 
                std::env::var("DISCORD_ID").unwrap()))
            .bearer_auth(token.unwrap())
            .json(&json!({
                "platform_name": "Lucio Surf League",
                "platform_username": activity.username,
                "metadata": {
                    "2_13": activity.title_new.as_ref().unwrap().clone() as i32
                }
            })).send().await;
    }
}

async fn set_title(ranks: &Vec<Ranking>, discord: &Discord, client: &Client, pool: &PgPool) {
    let token = get_access_token(discord, client, pool).await;
    if token.is_err() {
        return;
    }
    let token = token.unwrap();

    let name = ranks.first().map(|v| v.username.clone());
    if name.is_none() {
        return;
    }
    let name = name.unwrap();
    // Somehow contruct the json payload from the vec
    let mut json = json!({
        "platform_name": "Lucio Surf League",
        "platform_username": name,
        "metadata": {}
    });
    for rank in ranks {
        json["metadata"]
            .as_object_mut()
            .map(|v| {
                v.insert(
                    rank.patch.clone().replace(".", "_"), 
                    (rank.title.clone() as i32).into(),
                );
            });
    }
    let _ = client
        .put(format!("https://discord.com/api/v10/users/@me/applications/{}/role-connection", 
            std::env::var("DISCORD_ID").unwrap()))
        .bearer_auth(&token)
        .json(&json).send().await;
}

async fn get_access_token(tokens: &Discord, client: &Client, pool: &PgPool) -> Result<String, ()> {
    if tokens.expires_at > Local::now() {
        return Ok(tokens.access.clone());
    }
    match client.post("https://discord.com/api/v10/oauth2/token")
        .form(&[("client_id", std::env::var("DISCORD_ID").unwrap()), 
            ("client_secret", std::env::var("DISCORD_SECRET").unwrap()),
            ("grant_type", "refresh_token".into()), 
            ("refresh_token", tokens.refresh.clone())])
        .send().await {
            Ok(res) => {
                let auth = res.json::<AuthRes>().await.or(Err(()))?;
                let _ = query(
                    r#"UPDATE discord
                    SET access = $1, refresh = $2, expires_at = $3
                    WHERE id = $4"#)
                .bind(auth.access_token.clone())
                .bind(auth.refresh_token)
                .bind(Local::now() + TimeDelta::seconds(auth.expires_in))
                .bind(tokens.id)
                .execute(pool)
                .await;
                Ok(auth.access_token)
            }
            Err(_) => Err(())
        }
}