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

// HORRIBLE HACK: sqlx does not allow custom types in records, so here we are.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Display, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type), sqlx(type_name = "title"), repr(i32))]
pub enum Title {
    #[strum(to_string = "No Title")]
    None = 1,
    #[strum(to_string = "Surfer")]
    Surfer = 2,
    #[strum(to_string = "Super Surfer")]
    SuperSurfer = 3,
    #[strum(to_string = "Epic Surfer")]
    EpicSurfer = 4,
    #[strum(to_string = "Legendary Surfer")]
    LegendarySurfer = 5,
    #[strum(to_string = "Mythic Surfer")]
    MythicSurfer = 6,
    #[strum(to_string = "Rank 1")]
    TopOne = 7,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RunFilters {
    pub user: Option<i64>,
    pub patch: Option<String>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub map: Option<String>,
    pub status: Option<String>,
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
            status: None,
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct SectionFilters {
    pub patch: Option<String>,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub map: Option<String>,
    pub status: Option<String>,
    pub sort: String,
    pub ascending: bool,
}

impl Default for SectionFilters {
    fn default() -> Self {
        Self { patch: None, layout: None, category: None, map: None, status: None, sort: "map".into(), ascending: true }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserFilters {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub permission: Vec<Permissions>,
    pub sort: String,
    pub ascending: bool,
}

impl Default for UserFilters {
    fn default() -> Self {
        Self { id: None, name: None, permission: vec![], sort: "name".into(), ascending: true }
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
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow, sqlx::Type), sqlx(no_pg_array))]
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

#[cfg(feature = "ssr")]
impl sqlx::postgres::PgHasArrayType for Rank {
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
pub enum Permissions {
    View = 0,
    Submit = 1,
    Delete = 2,
    Trusted = 3,
    Verify = 4,
    ManageRuns = 5,
    ManageSections = 6,
    ManageUsers = 7,
    Administrator = 63,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub bio: Option<String>,
    pub pfp: String,
    pub created_at: DateTime<Local>,
    pub ranks: Vec<Rank>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
            username: "Guest".into(),
            bio: None,
            pfp: "default".into(),
            ranks: vec![],
            created_at: DateTime::default(),
        }
    }
}

impl From<RestrictedUser> for User {
    fn from(value: RestrictedUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
            bio: value.bio,
            pfp: value.pfp,
            ranks: value.ranks,
            created_at: value.created_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct RestrictedUser {
    pub id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub bio: Option<String>,
    pub pfp: String,
    pub ranks: Vec<Rank>,
    pub permission: BitVec,
    pub created_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PartialRestrictedUser {
    pub id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "name"))]
    pub username: String,
    pub bio: Option<String>,
    pub pfp: String,
    pub permission: BitVec,
    pub created_at: DateTime<Local>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow, sqlx::Type), sqlx(type_name = "RECORD"))]
pub struct PartialUser {
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow, sqlx::Type), sqlx(no_pg_array))]
pub struct Rank {
    pub id: i32,
    pub patch: String,
    pub layout: Option<String>,
    pub category: Option<String>,
    pub title: Title,
    pub rank: i32,
    pub rating: f64,
    pub percentage: f64,
    pub points: f64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Default for RestrictedUser {
    fn default() -> Self {
        let permissions = BitVec::from_elem(64, false);

        Self {
            id: -1,
            username: "Guest".into(),
            bio: None,
            permission: permissions,
            ranks: Vec::new(),
            pfp: "default".into(),
            created_at: DateTime::default(),
        }
    }
}

impl Default for PartialRestrictedUser {
    fn default() -> Self {
        let permissions = BitVec::from_elem(64, false);

        Self {
            id: -1,
            username: "Guest".into(),
            bio: None,
            permission: permissions,
            pfp: "default".into(),
            created_at: DateTime::default(),
        }
    }
}

pub trait GetPermissions {
    fn permissions(&self) -> &BitVec;
}

pub trait UserPermissions {
    fn has(&self, perm: &Permissions) -> bool;
    fn has_any(&self, perms: &[Permissions]) -> bool;
    fn has_all(&self, perms: &[Permissions]) -> bool;
}

impl GetPermissions for PartialRestrictedUser {
    fn permissions(&self) -> &BitVec {
        &self.permission
    }
}

impl GetPermissions for RestrictedUser {
    fn permissions(&self) -> &BitVec {
        &self.permission
    }
}

impl<T: GetPermissions> UserPermissions for T {
    fn has(&self, perm: &Permissions) -> bool {
        self.permissions().get(Permissions::Administrator as usize).is_some_and(|v| v)
            || self.permissions().get(*perm as usize).is_some_and(|v| v == true)
    }
    fn has_any(&self, perms: &[Permissions]) -> bool {
        if self.permissions().get(Permissions::Administrator as usize).is_some_and(|v| v) {
            return true;
        }
        for perm in perms {
            if self.permissions().get(*perm as usize).is_some_and(|v| v) {
                return true;
            }
        }
        return false;
    }
    fn has_all(&self, perms: &[Permissions]) -> bool {
        if self.permissions().get(Permissions::Administrator as usize).is_some_and(|v| v) {
            return true;
        }
        for perm in perms {
            if self.permissions().get(*perm as usize).is_none_or(|v| !v) {
                return false;
            }
        }
        return true;
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
