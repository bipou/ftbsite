use serde::Deserialize;
use surrealdb::types::SurrealValue;

#[derive(Debug, Deserialize, SurrealValue)]
pub(crate) struct CountResult {
    pub(crate) count: u64,
}

pub mod analysis_db;
pub mod auth;
pub mod captcha;
pub mod category_db;
pub mod db;
pub mod email;
pub mod football_db;
pub mod markdown;
pub mod topic_db;
pub mod upload;
pub mod user_db;
