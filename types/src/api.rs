use std::collections::HashSet;

use chrono::{DateTime, Local};
use leptos::{
    prelude::{FromServerFnError, ServerFnErrorErr},
    server_fn::codec::JsonEncoding,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
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
    pub user: Option<i64>,
    pub patch: Option<String>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub map: Option<String>,
    pub faster: Option<Decimal>,
    pub slower: Option<Decimal>,
    pub before: Option<DateTime<Local>>,
    pub after: Option<DateTime<Local>>,
    pub sort: String,
    pub ascending: bool,
}

impl Default for RunFilters {
    fn default() -> Self {
        Self {
            user: None,
            patch: None,
            layout: None,
            category: None,
            map: None,
            faster: None,
            slower: None,
            before: None,
            after: None,
            sort: String::from("created_at"),
            ascending: false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ActivityFilters {
    pub event: Option<String>,
    pub user: Option<i64>,
    pub patch: Option<String>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub before: Option<DateTime<Local>>,
    pub after: Option<DateTime<Local>>,
    pub sort: String,
    pub ascending: bool,
}

impl Default for ActivityFilters {
    fn default() -> Self {
        Self {
            event: None,
            user: None,
            patch: None,
            layout: None,
            category: None,
            before: None,
            after: None,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(type_name = "permissions"))]
pub enum Permissions {
    View,
    Submit,
    Trusted,
    Delete,
    Verify,
    ManageRuns,
    ManageUsers,
    Administrator,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub bio: Option<String>,
    pub pfp: String,
    pub ranks: Vec<Rank>,
    pub permissions: HashSet<Permissions>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Rank {
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
    pub percentage: f64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Default for User {
    fn default() -> Self {
        let permissions = HashSet::new();

        Self {
            id: -1,
            username: "Guest".into(),
            bio: None,
            permissions,
            ranks: Vec::new(),
            pfp: "default".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Discord {
    #[serde(rename = "username")]
    pub name: String,
    #[serde(rename = "id")]
    pub snowflake: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordUpdate {
    pub old: String,
    pub new: String,
}
