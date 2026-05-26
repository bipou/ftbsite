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
    summary: String,
    content: String,
    #[serde(default)]
    ai_model: String,
    generated_at: Sdt,
    status: i8,
}

fn analysis_into(d: AnalysisDoc) -> FootballAnalysis {
    let content_html = render_md(&d.content);
    FootballAnalysis {
        id: common::rid_str(&d.id),
        football_id: common::rid_str(&d.football_id),
        user_id: d.user_id.as_ref().map(common::rid_str),
        summary: d.summary,
        content: d.content,
        content_html,
        ai_model: d.ai_model,
        generated_at: common::ymdhmsz8(&d.generated_at),
        status: d.status,
    }
}

/// 获取某场比赛的所有分析文章
pub async fn get_analyses_by_football_id(rid: &RecordId) -> Result<Vec<FootballAnalysis>, String> {
    let mut res = get_db()
        .query(
            "SELECT * FROM footballs_analyses WHERE football_id = $fid ORDER BY generated_at DESC",
        )
        .bind(("fid", rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<AnalysisDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(analysis_into).collect())
}
