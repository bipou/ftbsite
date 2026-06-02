use crate::shared::common::rid_str;
use serde::Deserialize;
use std::collections::HashMap;
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::Category;
use crate::server::db::get_db;

#[derive(Debug, Deserialize, SurrealValue)]
struct CategoryDoc {
    id: RecordId,
    name: HashMap<String, String>,
    level: u8,
    #[serde(default)]
    pinned: Option<bool>,
}

pub async fn get_categories() -> Result<Vec<Category>, String> {
    let mut res = get_db()
        .query("SELECT * FROM categories ORDER BY level ASC")
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs
        .into_iter()
        .map(|d| Category {
            id: rid_str(&d.id),
            name: d.name,
            level: d.level,
            pinned: d.pinned,
        })
        .collect())
}
