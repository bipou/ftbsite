use crate::shared::common::{into_rid, rid_str};
use serde::Deserialize;
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::Topic;
use crate::server::db::get_db;

// ── Document types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct TopicDoc {
    id: RecordId,
    name: String,
    quotes: i64,
}

// ── Conversion ─────────────────────────────────────────────────────────────────

fn into_topic(d: TopicDoc) -> Topic {
    Topic {
        id: rid_str(&d.id),
        name: d.name,
        quotes: d.quotes,
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

pub async fn get_keywords_by_user_id(user_rid: &RecordId) -> Result<Vec<Topic>, String> {
    let mut res = get_db()
        .query("SELECT VALUE topic_id FROM topics_rel WHERE user_id = $uid AND football_id IS NONE")
        .bind(("uid", user_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let topic_rids: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for rid in &topic_rids {
        let tid = rid_str(rid);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }

    if tids.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_topic).collect())
}

pub async fn get_topics_by_user_id(user_rid: &RecordId) -> Result<Vec<Topic>, String> {
    let mut res = get_db()
        .query("SELECT VALUE topic_id FROM topics_rel WHERE user_id = $uid")
        .bind(("uid", user_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let topic_rids: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for rid in &topic_rids {
        let tid = rid_str(rid);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }

    if tids.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_topic).collect())
}

pub async fn create_topics_from_names(names: &str) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    for raw in names.split(|c: char| c == ',' || c == ' ' || c == '\n') {
        let name = raw.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }

        // Check if topic already exists
        let mut res = get_db()
            .query("SELECT * FROM topics WHERE name = $name")
            .bind(("name", name.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;

        if let Some(doc) = docs.first() {
            // Increment quotes on existing topic
            get_db()
                .query("UPDATE $rid SET quotes += 1")
                .bind(("rid", doc.id.clone()))
                .await
                .map_err(|e| e.to_string())?;
            ids.push(rid_str(&doc.id));
        } else {
            // Create new topic
            let mut res = get_db()
                .query("CREATE topics CONTENT { name: $name, quotes: 1 }")
                .bind(("name", name))
                .await
                .map_err(|e| e.to_string())?;
            let new_docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
            if let Some(doc) = new_docs.first() {
                ids.push(rid_str(&doc.id));
            }
        }
    }

    Ok(ids)
}

/// 批量回填足球条目的 topics——与 DB 子查询解耦
pub async fn batch_fill_football_topics(
    items: &mut [crate::models::Football],
) -> Result<(), String> {
    let fids: Vec<RecordId> = items.iter().map(|f| into_rid(&f.id, "footballs")).collect();
    if fids.is_empty() {
        return Ok(());
    }
    // 一次查询拿到所有 football → topic 关联
    let mut rel_res = get_db()
        .query("SELECT football_id, topic_id FROM topics_rel WHERE football_id IN $fids")
        .bind(("fids", fids))
        .await
        .map_err(|e| e.to_string())?;
    let rels: Vec<RelRow> = rel_res.take(0).map_err(|e| e.to_string())?;
    if rels.is_empty() {
        return Ok(());
    }
    // 去重收集所有 topic_id
    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = rid_str(&r.topic_id);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }
    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut top_res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = top_res.take(0).map_err(|e| e.to_string())?;
    let topics: Vec<Topic> = docs.into_iter().map(into_topic).collect();
    // 按 football_id 分组回填
    for item in items.iter_mut() {
        let fid = into_rid(&item.id, "footballs");
        let tids: Vec<String> = rels
            .iter()
            .filter(|r| r.football_id == fid)
            .map(|r| rid_str(&r.topic_id))
            .collect();
        item.topics = topics
            .iter()
            .filter(|t| tids.contains(&t.id))
            .cloned()
            .collect();
    }
    Ok(())
}

// ── Relation row for batch query ────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct RelRow {
    football_id: RecordId,
    topic_id: RecordId,
}

pub async fn link_topics_to_football(
    football_id: &str,
    topic_ids: &[String],
) -> Result<(), String> {
    let fid = into_rid(football_id, "footballs");
    for tid in topic_ids {
        let topic_rid = into_rid(tid, "topics");
        let mut res = get_db()
            .query("SELECT VALUE id FROM topics_rel WHERE football_id = $fid AND topic_id = $tid")
            .bind(("fid", fid.clone()))
            .bind(("tid", topic_rid.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let rel_ids: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;
        if rel_ids.is_empty() {
            get_db()
                .query("CREATE topics_rel CONTENT { football_id: $fid, topic_id: $tid }")
                .bind(("fid", fid.clone()))
                .bind(("tid", topic_rid.clone()))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn link_topics_to_user(user_id: &str, topic_ids: Vec<String>) -> Result<(), String> {
    let user_rid = into_rid(user_id, "users");
    for tid in &topic_ids {
        let topic_rid = into_rid(tid, "topics");

        // Check if relation already exists
        let mut res = get_db()
            .query("SELECT VALUE id FROM topics_rel WHERE user_id = $uid AND topic_id = $tid AND football_id IS NONE")
            .bind(("uid", user_rid.clone()))
            .bind(("tid", topic_rid.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let rel_ids: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;

        if rel_ids.is_empty() {
            get_db()
                .query("CREATE topics_rel CONTENT { user_id: $uid, topic_id: $tid }")
                .bind(("uid", user_rid.clone()))
                .bind(("tid", topic_rid.clone()))
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
