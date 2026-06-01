use crate::models::FootballAnalysis;
use crate::server::db::get_db;
use crate::server::markdown::render_md;
use crate::shared::common;
use serde::Deserialize;
use surrealdb::types::{Datetime as Sdt, RecordId, SurrealValue};

#[derive(Debug, Deserialize, SurrealValue)]
struct AnalysisDoc {
    id: RecordId,
    football_id: RecordId,
    #[serde(default)]
    user_id: Option<RecordId>,
    #[serde(default)]
    summary: Option<String>,
    content: String,
    #[serde(default)]
    ai_model: String,
    #[serde(default)]
    generated_at: Option<Sdt>,
    status: i8,
}

/// 获取某场比赛的所有分析文章
pub async fn get_analyses_by_football_id(rid: &RecordId) -> Result<Vec<FootballAnalysis>, String> {
    let mut res = get_db()
        .query("SELECT * FROM footballs_analyses WHERE football_id = $fid ORDER BY id DESC")
        .bind(("fid", rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<AnalysisDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs
        .into_iter()
        .map(|d| FootballAnalysis {
            id: common::rid_str(&d.id),
            football_id: common::rid_str(&d.football_id),
            user_id: d.user_id.as_ref().map(common::rid_str),
            summary: d.summary,
            content: d.content.clone(),
            content_html: render_md(&d.content),
            ai_model: d.ai_model,
            status: d.status,
        })
        .collect())
}

/// 插入分析正文
pub async fn insert_analysis(
    football_id: &str,
    content: &str,
    user_id: &str,
    summary: &str,
) -> Result<(), String> {
    let fid = common::into_rid(football_id, "footballs");
    get_db()
        .query(
            "CREATE footballs_analyses SET football_id = $fid, content = $content, user_id = $uid, summary = $summary, ai_model = '', status = 1"
        )
        .bind(("fid", fid))
        .bind(("content", content.to_string()))
        .bind(("uid", common::into_rid(user_id, "users")))
        .bind(("summary", summary.to_string()))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
