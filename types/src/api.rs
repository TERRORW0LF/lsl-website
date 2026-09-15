use bit_vec::BitVec;
use chrono::{DateTime, Local};
use leptos::{
    prelude::{FromServerFnError, ServerFnErrorErr},
    server_fn::codec::JsonEncoding,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error, EnumString, Serialize, Deserialize)]
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
            Registration(v) | MiddlewareError(v) | ServerError(v) | Args(v) | MissingArg(v) | Response(v) => {
                ApiError::ServerError(v)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Display, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(type_name = "title"))]
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
pub struct PartialSection {
    #[cfg_attr(feature = "ssr", sqlx(rename = "section_id"))]
    pub id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub submittable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Section {
    pub id: i32,
    pub patch: String,
    pub layout: String,
    pub category: String,
    pub map: String,
    pub code: String,
    pub submittable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Run {
    pub id: i32,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub section: PartialSection,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub user: PartialUser,
    pub time: Decimal,
    pub proof: String,
    pub yt_id: Option<String>,
    pub verified: bool,
    pub is_pb: bool,
    pub is_wr: bool,
    pub created_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), derive(sqlx::FromRow), sqlx(no_pg_array))]
pub struct PartialRun {
    pub id: i32,
    pub section_id: i32,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub user: PartialUser,
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
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub section: PartialSection,
    pub runs: Vec<PartialRun>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Ranking {
    pub id: i32,
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub user: PartialUser,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
    pub percentage: f64,
    pub points: f64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(no_pg_array))]
pub struct PartialRanking {
    pub id: i32,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub user: PartialUser,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
    pub percentage: f64,
    pub points: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ComboRanking {
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub rankings: Vec<PartialRanking>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Activity {
    pub id: i32,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub user: PartialUser,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(type_name = "permissions"))]
pub enum Permissions {
    View = 0,
    Submit = 1,
    Trusted = 2,
    Delete = 3,
    Verify = 4,
    ManageRuns = 5,
    ManageSections = 6,
    ManageUsers = 7,
    Administrator = 63,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub bio: Option<String>,
    pub pfp: String,
    pub ranks: Vec<Rank>,
    pub permissions: BitVec,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow), derive(sqlx::Type), sqlx(type_name = "RECORD"))]
pub struct PartialUser {
    pub user_id: i64,
    pub username: String,
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
        let permissions = BitVec::from_bytes(&[0b10000000, 0b0, 0b0, 0b0]);

        Self { id: -1, username: "Guest".into(), bio: None, permissions, ranks: Vec::new(), pfp: "default".into() }
    }
}

pub trait UserPermissions {
    fn has(&self, perm: &Permissions) -> bool;
}

impl UserPermissions for User {
    fn has(&self, perm: &Permissions) -> bool {
        self.permissions.get(Permissions::Administrator as usize).is_some_and(|v| v == true)
            || self.permissions.get(*perm as usize).is_some_and(|v| v == true)
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
