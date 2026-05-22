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
}

fn into_category(d: CategoryDoc) -> Category {
    Category {
        id: rid_str(&d.id),
        name: d.name,
        level: d.level,
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

pub async fn get_categories_by_levels(levels: &[u8]) -> Result<Vec<Category>, String> {
    if levels.is_empty() {
        return Ok(vec![]);
    }
    let q = format!(
        "SELECT * FROM categories WHERE level IN [{}] ORDER BY level ASC",
        levels
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_category).collect())
}
