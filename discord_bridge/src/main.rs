use chrono::{DateTime, Local, TimeDelta};
use log::debug;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::{PgConnectOptions, PgListener};
use sqlx::prelude::FromRow;
use sqlx::{query, query_as, PgPool};
use urlencoding::encode;
use strum::Display;

#[derive(Clone, Display, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "title")]
pub enum Title {
    #[strum(to_string = "No Title")]
    None = 0,
    #[strum(to_string = "Surfer")]
    Surfer = 1,
    #[strum(to_string = "Super Surfer")]
    SuperSurfer = 2,
    #[strum(to_string = "Epic Surfer")]
    EpicSurfer = 3,
    #[strum(to_string = "Legendary Surfer")]
    LegendarySurfer = 4,
    #[strum(to_string = "Mythic Surfer")]
    MythicSurfer = 5,
    #[strum(to_string = "Rank 1")]
    TopOne = 6,
}

#[derive(Clone, FromRow)]
struct Activity {
    id: i32,
    user_id: i64,
    name: String,
    title_old: Option<Title>,
    title_new: Option<Title>,
    rank_old: Option<i32>,
    rank_new: Option<i32>,
    created_at: DateTime<Local>,
}

#[derive(Clone, FromRow)]
struct Run {
    id: i32,
    user_id: i64,
    name: String,
    section_id: i32,
    patch: String,
    layout: String,
    category: String,
    map: String,
    time: Decimal,
    yt_id: String,
    is_pb: bool,
    is_wr: bool,
    created_at: DateTime<Local>,
}

#[derive(Clone, FromRow)]
struct PbRun {
    id: i32,
    time: Decimal,
    yt_id: String,
    created_at: DateTime<Local>,
}

#[derive(Clone, FromRow)]
struct WrRun {
    id: i32,
    name: String,
    time: Decimal,
    yt_id: String,
    created_at: DateTime<Local>,
}

#[derive(Clone, FromRow)]
struct Discord {
    id: i32,
    access: String,
    refresh: String,
    expires_at: DateTime<Local>,
}

#[derive(Deserialize)]
struct AuthRes {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_env().expect("couldn't initialize logging");

    let submit_client = Client::new();
    let activity_client = Client::new();

    let connect_opts = PgConnectOptions::new()
        .database(&std::env::var("PG_DB").unwrap())
        .username(&std::env::var("PG_USER").unwrap())
        .password(&std::env::var("PG_PASS").unwrap())
        .host(&std::env::var("PG_HOST").unwrap())
        .port(std::env::var("PG_PORT").unwrap().parse::<u16>().unwrap());

    let submit_pool = PgPool::connect_with(connect_opts).await.unwrap();
    let activity_pool = submit_pool.clone();
    let submit = tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&submit_pool).await.unwrap();
        listener.listen("submit").await.unwrap();
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let run = query_as::<_, Run>(
                        r#"SELECT r.id, r.user_id, u.name, r.section_id, s.patch, s.layout, s.category, 
                            s.map, r.time, r.yt_id, r.is_pb, r.is_wr, r.created_at
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
                                let old = query_as::<_, WrRun>(
                                    r#"SELECT r.id, u.name, r.time, r.yt_id, r.created_at
                                    FROM run r
                                    INNER JOIN "user" u ON r.user_id = u.id
                                    WHERE section_id = $1
                                    ORDER BY time ASC, created_at ASC
                                    LIMIT 1;"#,
                                )
                                .bind(r.section_id)
                                .fetch_optional(&submit_pool)
                                .await
                                .ok()
                                .flatten();
                                send_wr(&r, &old, &submit_client).await;
                            } else if r.is_pb {
                                let old = query_as::<_, PbRun>(
                                    r#"SELECT id, time, yt_id, created_at
                                    FROM run
                                    WHERE section_id = $1 AND user_id = $2
                                    ORDER BY time ASC, created_at ASC
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
                        r#"SELECT a.id, a.user_id, u.name, a.title_old, a.title_new, 
                            a.rank_old, a.rank_new, a.created_at
                        FROM activity a
                        INNER JOIN "user" u ON a.user_id = u.id
                        WHERE a.id = $1::integer;"#,
                    )
                    .bind(notification.payload())
                    .fetch_one(&activity_pool)
                    .await;

                    match activity {
                        Ok(a) => {
                            if let Some(ref t_new) = a.title_new && let Some(ref t_old) = a.title_old {
                                send_title(&a.name, t_new, t_old, &activity_client).await;
                            } else if let Some(ref r_new) = a.rank_new && let Some(ref r_old) = a.rank_old {
                                send_rank(&a.name, r_new, r_old, &activity_client).await;
                            } else {
                                send_join(&a.name, &activity_client).await;
                            }
                            if let Some(ref t_new) = a.title_new {
                                let discord = query_as::<_, Discord>(
                                    r#"SELECT id, access, refresh, expires_at
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

    submit.await.unwrap();
    activity.await.unwrap();
}

async fn send_pb(new: &Run, old: &Option<PbRun>, client: &Client) {
    let _ = client.post(std::env::var("PB_WEBHOOK").unwrap()).json(&json!({ 
        "embeds": [{
            "color": 16764928,
            "title": format!("New Personal Best by {}", new.name),
            "url": format!("https://youtube.com/watch?v={}", new.yt_id),
            "thumbnail": { "url": format!("https:lucio.surf/cdn/maps/{}.jpg", encode(&new.map))},
            "description": format!("Patch: *{}*\nLayout: *{}*\nCategory: *{}*\nMap: *{}*", 
                new.patch, new.layout, new.category, new.map),
            "fields": [{
                "name": "New PB",
                "value": format!("Time: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                    new.time.to_string(), new.yt_id, new.created_at.timestamp()),
                "inline": true
            },
            {
                "name": "Old PB",
                "value": match old {
                    Some(o) => format!("Time: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                        o.time.to_string(), o.yt_id, o.created_at.timestamp()),
                    None => "Time: *none*\nProof: *none*\nDate: *none".into(),
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
        "content": format!("https://youtube.com/watch?v={}", new.yt_id)
    })).send().await;
}

async fn send_wr(new: &Run, old: &Option<WrRun>, client: &Client) {
    let _ = client.post(std::env::var("WR_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": 7798548,
            "title": format!("New World Record by {}", new.name),
            "url": format!("https://youtube.com/watch?v={}", new.yt_id),
            "thumbnail": { "url": format!("https://lucio.surf/cdn/maps/{}.jpg", encode(&new.map))},
            "description": format!("Patch: *{}*\nLayout: *{}*\nCategory: *{}*\nMap: *{}*", 
                new.patch, new.layout, new.category, new.map),
            "fields": [{
                "name": "New Record",
                "value": format!("User: *{}*\nTime: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                    new.name, new.time.to_string(), new.yt_id, new.created_at.timestamp()),
                "inline": true
            },
            {
                "name": "Old Record",
                "value": match old {
                    Some(o) => format!("User: *{}*\nTime: *{}*\nProof: *[link](https://youtube.com/watch?v={})*\nDate: *<t:{}>*", 
                        o.name, o.time.to_string(), o.yt_id, o.created_at.timestamp()),
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
        "content": format!("https://youtube.com/watch?v={}", new.yt_id)
    })).send().await;
}

async fn send_title(name: &String, new: &Title, old: &Title, client: &Client) {
    let _ = client.post(std::env::var("ACTIVITY_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": if new > old { 7798548 } else { 12064000 },
            "title": "Title update",
            "description": format!("User: *{name}*"),
            "fields": [{
                "name": "New Title",
                "value": new.to_string(),
                "inline": true
            },
            {
                "name": "Old Title",
                "value": old.to_string(),
                "inline": true
            }]
        }]
    })).send().await;
}

async fn send_rank(name: &String, new: &i32, old: &i32, client: &Client) {}

async fn send_join(name: &String, client: &Client) {
    let _ = client.post(std::env::var("ACTIVITY_WEBHOOK").unwrap()).json(&json!({
        "embeds": [{
            "color": 1342207,
            "title": format!("{name} joined the leaderboards!")
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
                "platform_username": activity.name,
                "metadata": {
                    "title": activity.title_new.as_ref().unwrap().clone() as i32
                }
            })).send().await;
    }
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