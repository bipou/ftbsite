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

fn into_category(d: CategoryDoc) -> Category {
    Category {
        id: rid_str(&d.id),
        name: d.name,
        level: d.level,
        pinned: d.pinned,
    }
}

pub async fn get_categories() -> Result<Vec<Category>, String> {
    let mut res = get_db()
        .query("SELECT * FROM categories ORDER BY level ASC")
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_category).collect())
}

pub async fn get_category_by_id(rid: &RecordId) -> Result<Option<Category>, String> {
    let doc: Option<CategoryDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;
    Ok(doc.map(into_category))
}

/// 批量取类别
pub async fn get_categories_batch(rids: &[&RecordId]) -> Result<HashMap<String, Category>, String> {
    if rids.is_empty() {
        return Ok(HashMap::new());
    }
    let in_clause = rids
        .iter()
        .map(|r| rid_str(r))
        .collect::<Vec<_>>()
        .join(", ");
    let q = format!("SELECT * FROM categories WHERE id IN [{}]", in_clause);
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs
        .into_iter()
        .map(|d| (rid_str(&d.id), into_category(d)))
        .collect())
}
