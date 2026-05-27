use crate::shared::common::{self, rid_str};
use crate::shared::constant;
use chrono::{Duration, FixedOffset, TimeZone, Timelike, Utc};
use serde::Deserialize;
use surrealdb::types::{Datetime as Sdt, RecordId, SurrealValue};

#[cfg(feature = "oth")]
use crate::models::{Calc, Line};
use crate::models::{Football, FootballEvent, FootballStats, FootballsResult, TeamLineup};
use crate::server::{analysis_db, category_db, db::get_db, topic_db};

// ── Datetime formatters ────────────────────────────────────────────────────────

fn mdhm(dt: &Sdt) -> String {
    dt.format("%m-%d %H:%M").to_string()
}

fn mdhm8(dt: &Sdt) -> String {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%m-%d %H:%M").to_string()
}

// ── Doc structs ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballDoc {
    id: RecordId,
    category_id: RecordId,
    season: String,
    home_team: String,
    away_team: String,
    kick_off_at: Sdt,
    created_at: Sdt,
    updated_at: Sdt,
    #[serde(default)]
    hits: Option<i64>,
    #[serde(default)]
    stars: Option<i64>,
    status: i8,
    s: Option<String>,
    wdl: Option<u8>,
    tg: Option<u8>,
    gd: Option<i8>,
    #[cfg(feature = "oth")]
    #[serde(default)]
    lines: Vec<LineDoc>,
    #[cfg(feature = "oth")]
    #[serde(default)]
    calcs: Vec<CalcDoc>,
    #[serde(default)]
    home_lineup: Option<TeamLineupDoc>,
    #[serde(default)]
    away_lineup: Option<TeamLineupDoc>,
    #[serde(default)]
    events: Option<Vec<FootballEventDoc>>,
    #[serde(default)]
    stats: Option<FootballStatsDoc>,
    #[serde(default)]
    article_title: Option<String>,
    #[serde(default)]
    ana_type: i8,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct LineupPlayerDoc {
    number: u8,
    name: String,
    position: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct TeamLineupDoc {
    formation: String,
    #[serde(default)]
    coach: Option<String>,
    starters: Vec<LineupPlayerDoc>,
    substitutes: Vec<LineupPlayerDoc>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballEventDoc {
    minute: u8,
    #[serde(default)]
    extra: Option<u8>,
    #[serde(rename = "type")]
    event_type: String,
    player: String,
    team: String,
    #[serde(default)]
    assist: Option<String>,
    #[serde(default)]
    player_out: Option<String>,
    note: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct SideStatsDoc {
    home: f32,
    away: f32,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct SideStatsIntDoc {
    home: u16,
    away: u16,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballStatsDoc {
    possession: SideStatsDoc,
    shots: SideStatsIntDoc,
    shots_on_target: SideStatsIntDoc,
    corners: SideStatsIntDoc,
    fouls: SideStatsIntDoc,
    offsides: SideStatsIntDoc,
    yellow_cards: SideStatsIntDoc,
    red_cards: SideStatsIntDoc,
    passes: SideStatsIntDoc,
    pass_accuracy: SideStatsDoc,
}

#[cfg(feature = "oth")]
#[derive(Debug, Deserialize, SurrealValue)]
struct LineDoc {
    win: f32,
    draw: f32,
    loss: f32,
    created_at: Sdt,
}

#[cfg(feature = "oth")]
#[derive(Debug, Deserialize, SurrealValue)]
struct CalcDoc {
    s: String,
    wdl: String,
    tg: String,
    gd: String,
    created_at: Sdt,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct CountResult {
    count: u64,
}

// ── Doc → Model ────────────────────────────────────────────────────────────────

fn lp_to(d: LineupPlayerDoc) -> crate::models::LineupPlayer {
    crate::models::LineupPlayer {
        number: d.number,
        name: d.name,
        position: d.position,
    }
}

fn tl_to(d: TeamLineupDoc) -> TeamLineup {
    TeamLineup {
        formation: d.formation,
        coach: d.coach,
        starters: d.starters.into_iter().map(lp_to).collect(),
        substitutes: d.substitutes.into_iter().map(lp_to).collect(),
    }
}

fn fe_to(d: FootballEventDoc) -> FootballEvent {
    FootballEvent {
        minute: d.minute,
        extra: d.extra,
        event_type: d.event_type,
        player: d.player,
        team: d.team,
        assist: d.assist,
        player_out: d.player_out,
        note: d.note,
    }
}

fn ss_to(d: SideStatsDoc) -> crate::models::SideStats {
    crate::models::SideStats {
        home: d.home,
        away: d.away,
    }
}

fn ssi_to(d: SideStatsIntDoc) -> crate::models::SideStatsInt {
    crate::models::SideStatsInt {
        home: d.home,
        away: d.away,
    }
}

fn fst_to(d: FootballStatsDoc) -> FootballStats {
    FootballStats {
        possession: ss_to(d.possession),
        shots: ssi_to(d.shots),
        shots_on_target: ssi_to(d.shots_on_target),
        corners: ssi_to(d.corners),
        fouls: ssi_to(d.fouls),
        offsides: ssi_to(d.offsides),
        yellow_cards: ssi_to(d.yellow_cards),
        red_cards: ssi_to(d.red_cards),
        passes: ssi_to(d.passes),
        pass_accuracy: ss_to(d.pass_accuracy),
    }
}

// ── oth-only helpers ───────────────────────────────────────────────────────────

#[cfg(feature = "oth")]
fn line_to(d: LineDoc) -> Line {
    Line {
        win: d.win,
        draw: d.draw,
        loss: d.loss,
        created_at: common::ymdhmsz8(&d.created_at),
    }
}

#[cfg(feature = "oth")]
fn calc_to(d: CalcDoc) -> Calc {
    Calc {
        s: d.s,
        wdl: d.wdl,
        tg: d.tg,
        gd: d.gd,
        created_at: common::ymdhmsz8(&d.created_at),
    }
}

#[cfg(feature = "oth")]
fn il_pair<T: Clone>(v: &[T]) -> Vec<T> {
    match v.len() {
        0 => vec![],
        1 => vec![v[0].clone()],
        n => vec![v[0].clone(), v[n - 1].clone()],
    }
}

// ── Enrich ─────────────────────────────────────────────────────────────────────

async fn enrich(doc: FootballDoc) -> Result<Football, String> {
    let fid = rid_str(&doc.id);
    #[cfg(feature = "oth")]
    let lines: Vec<Line> = doc.lines.into_iter().map(line_to).collect();
    #[cfg(feature = "oth")]
    let calcs: Vec<Calc> = doc.calcs.into_iter().map(calc_to).collect();
    let topics = topic_db::get_topics_by_football_id(&doc.id).await?;
    let category = category_db::get_category_by_id(&doc.category_id).await?;
    let analyses = analysis_db::get_analyses_by_football_id(&doc.id).await?;
    let summary = if doc.ana_type == 2 {
        analyses
            .iter()
            .find(|a| a.status == 1)
            .map(|a| a.summary.clone())
    } else {
        None
    };

    Ok(Football {
        id: fid,
        category_id: rid_str(&doc.category_id),
        season: doc.season,
        home_team: doc.home_team,
        away_team: doc.away_team,
        kick_off_at_mdhm: mdhm(&doc.kick_off_at),
        kick_off_at_mdhm8: mdhm8(&doc.kick_off_at),
        created_at: common::ymdhmsz8(&doc.created_at),
        updated_at: common::ymdhmsz8(&doc.updated_at),
        hits: doc.hits.unwrap_or(0).max(0) as u64,
        stars: doc.stars.unwrap_or(0).max(0) as u64,
        status: doc.status,
        #[cfg(feature = "oth")]
        il_odds: il_pair(&lines),
        #[cfg(feature = "oth")]
        all_odds: lines,
        #[cfg(feature = "oth")]
        il_calcs: il_pair(&calcs),
        #[cfg(feature = "oth")]
        all_calcs: calcs,
        home_lineup: doc.home_lineup.map(tl_to),
        away_lineup: doc.away_lineup.map(tl_to),
        events: doc
            .events
            .unwrap_or_default()
            .into_iter()
            .map(fe_to)
            .collect(),
        stats: doc.stats.map(fst_to),
        summary,
        analyses,
        article_title: doc.article_title,
        ana_type: doc.ana_type as u8,
        result_s: doc.s,
        result_wdl: doc.wdl,
        result_tg: doc.tg,
        result_gd: doc.gd,
        category,
        topics,
    })
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// 首页取最新 12 篇（limit 为总数上限，Rust 层分组截取）
pub async fn get_home_footballs(limit: i64) -> Result<Vec<Football>, String> {
    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE status >= 1 ORDER BY created_at DESC LIMIT $limit")
        .bind(("limit", limit))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for d in docs {
        out.push(enrich(d).await?);
    }
    Ok(out)
}

pub async fn get_footballs_in_position(
    position: &str,
    limit: i64,
) -> Result<Vec<Football>, String> {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_utc = Utc::now();
    let now8 = now_utc.with_timezone(&tz8);
    let cutoff: i64 = if now8.hour() >= 11 { 0 } else { 1 };
    let day_off = match position {
        "jt" => -cutoff,
        "zt" => -(cutoff + 1),
        _ => return Err("position must be 'jt' or 'zt'".into()),
    };

    let target = now8.date_naive() + Duration::days(day_off);
    let start_l = tz8
        .from_local_datetime(&target.and_hms_opt(11, 0, 0).unwrap())
        .single()
        .ok_or("ambiguous datetime")?;
    let end_l = start_l + Duration::days(1);

    let start_utc = start_l.with_timezone(&Utc);
    let end_utc = end_l.with_timezone(&Utc);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE kick_off_at >= $start AND kick_off_at < $end AND status >= 0 ORDER BY kick_off_at DESC LIMIT $limit")
        .bind(("start", start_utc))
        .bind(("end", end_utc))
        .bind(("limit", limit))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for d in docs {
        out.push(enrich(d).await?);
    }
    Ok(out)
}

pub async fn get_football_by_id(rid: &RecordId) -> Result<Option<Football>, String> {
    let doc: Option<FootballDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;
    match doc {
        Some(d) => Ok(Some(enrich(d).await?)),
        None => Ok(None),
    }
}

pub async fn get_random_football_id() -> Result<Option<String>, String> {
    let mut res = get_db()
        .query("SELECT VALUE id FROM footballs WHERE status >= 1 AND kick_off_at >= time::now() - 1d ORDER BY rand() LIMIT 1")
        .await
        .map_err(|e| e.to_string())?;
    let ids: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;
    Ok(ids.into_iter().next().map(|id| rid_str(&id)))
}

pub async fn get_footballs(
    from: i64,
    status_min: i8,
    status_max: i8,
) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;
    let mut cres = get_db()
        .query("SELECT count() FROM footballs WHERE status >= $min AND status <= $max GROUP ALL")
        .bind(("min", status_min))
        .bind(("max", status_max))
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE status >= $min AND status <= $max ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip")
        .bind(("min", status_min))
        .bind(("max", status_max))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_by_category(
    category_rid: &RecordId,
    from: i64,
) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;

    let mut cres = get_db()
        .query("SELECT count() FROM footballs WHERE category_id = $cid AND status >= 1 GROUP ALL")
        .bind(("cid", category_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE category_id = $cid AND status >= 1 ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip")
        .bind(("cid", category_rid.clone()))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_by_topic(
    topic_rid: &RecordId,
    from: i64,
) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;

    let mut rel_res = get_db()
        .query(
            "SELECT VALUE football_id FROM topics_rel WHERE topic_id = $tid AND football_id IS NOT NONE",
        )
        .bind(("tid", topic_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let fids_raw: Vec<RecordId> = rel_res.take(0).map_err(|e| e.to_string())?;

    let mut fids: Vec<String> = Vec::new();
    for rid in &fids_raw {
        let fid = rid_str(rid);
        if !fids.contains(&fid) {
            fids.push(fid);
        }
    }

    let total = fids.len() as u64;
    let skip = ((from - 1) * ps).max(0) as usize;
    let page_fids: Vec<String> = fids.into_iter().skip(skip).take(ps as usize).collect();

    if page_fids.is_empty() {
        return Ok(FootballsResult {
            page_info: common::make_page_info(from, ps, total),
            items: vec![],
        });
    }

    let q = format!(
        "SELECT * FROM footballs WHERE id IN [{}] AND status >= 1 ORDER BY kick_off_at DESC",
        page_fids
            .iter()
            .map(|id| format!("{}", id))
            .collect::<Vec<_>>()
            .join(",")
    );

    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_admin(from: i64) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;

    let mut cres = get_db()
        .query("SELECT count() FROM footballs GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs ORDER BY updated_at DESC LIMIT $ps START $skip")
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn update_football_status(rid: &RecordId, status: i8) -> Result<(), String> {
    get_db()
        .query("UPDATE $rid SET status = $status, updated_at = time::now()")
        .bind(("rid", rid.clone()))
        .bind(("status", status))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn increment_hits(rid: &RecordId) -> Result<(), String> {
    get_db()
        .query("UPDATE $rid SET hits += 1")
        .bind(("rid", rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 用户发表文章：插入轻量 footballs 壳
pub async fn insert_article(title: &str, category_id: &str, status: i8) -> Result<String, String> {
    let rid = common::into_rid(category_id, "categories");
    let mut res = get_db()
        .query(
            "CREATE footballs SET article_title = $title, category_id = $cid, ana_type = 0, status = $status, home_team = '', away_team = '', season = '', kick_off_at = time::now(), created_at = time::now(), updated_at = time::now()"
        )
        .bind(("title", title.to_string()))
        .bind(("cid", rid))
        .bind(("status", status))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<RecordId> = res.take(0).map_err(|e| e.to_string())?;
    docs.into_iter()
        .next()
        .map(|id| rid_str(&id))
        .ok_or_else(|| "创建失败".to_string())
}
