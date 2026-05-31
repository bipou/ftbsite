use crate::shared::common::{self, rid_str};
use crate::shared::constant;
use chrono::FixedOffset;
use serde::Deserialize;
use surrealdb::types::{Datetime as Sdt, RecordId, SurrealValue};

#[cfg(feature = "oth")]
use crate::models::{Calc, Line};
use crate::models::{Football, FootballEvent, FootballStats, FootballsResult, TeamLineup};
use crate::server::{analysis_db, category_db, db::get_db, topic_db};

fn format_date_utc(dt: &Sdt) -> String {
    dt.format("%m-%d %H:%M").to_string()
}
fn format_date_utc8(dt: &Sdt) -> String {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%m-%d %H:%M").to_string()
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballDoc {
    id: RecordId,
    category_id: RecordId,
    #[serde(default)]
    season: Option<String>,
    #[serde(default)]
    home_team: Option<String>,
    #[serde(default)]
    away_team: Option<String>,
    #[serde(default)]
    kick_off_at: Option<Sdt>,
    created_at: Sdt,
    updated_at: Sdt,
    #[serde(default)]
    hits: Option<i64>,
    status: i8,
    s: Option<String>,
    wdl: Option<u8>,
    tg: Option<u8>,
    gd: Option<i8>,
    #[cfg(feature = "oth")]
    #[serde(default)]
    lines: Option<Vec<LineDoc>>,
    #[cfg(feature = "oth")]
    #[serde(default)]
    calcs: Option<Vec<CalcDoc>>,
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

#[cfg(feature = "oth")]
const FL: &str = "id, category_id, season, home_team, away_team, kick_off_at, created_at, updated_at, hits, status, s, wdl, tg, gd, lines, calcs, article_title, ana_type";
#[cfg(not(feature = "oth"))]
const FL: &str = "id, category_id, season, home_team, away_team, kick_off_at, created_at, updated_at, hits, status, s, wdl, tg, gd, article_title, ana_type";

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

#[cfg(feature = "oth")]
fn first_last<T: Clone>(v: &[T]) -> Vec<T> {
    match v.len() {
        0 => vec![],
        1 => vec![v[0].clone()],
        n => vec![v[0].clone(), v[n - 1].clone()],
    }
}

fn build_football(
    doc: FootballDoc,
    #[cfg(feature = "oth")] lines: Vec<Line>,
    #[cfg(feature = "oth")] calcs: Vec<Calc>,
    topics: Vec<crate::models::Topic>,
    category: Option<crate::models::Category>,
    summary: Option<String>,
    analyses: Vec<crate::models::FootballAnalysis>,
) -> Football {
    let fid = rid_str(&doc.id);
    let lp = |p: &LineupPlayerDoc| crate::models::LineupPlayer {
        number: p.number,
        name: p.name.clone(),
        position: p.position.clone(),
    };
    Football {
        id: fid,
        category_id: rid_str(&doc.category_id),
        season: doc.season,
        home_team: doc.home_team,
        away_team: doc.away_team,
        kick_off_at_mdhm: doc.kick_off_at.as_ref().map(|k| format_date_utc(k)),
        kick_off_at_mdhm8: doc.kick_off_at.as_ref().map(|k| format_date_utc8(k)),
        created_at: common::ymdhmsz8(&doc.created_at),
        updated_at: common::ymdhmsz8(&doc.updated_at),
        hits: doc.hits.unwrap_or(0).max(0) as u64,
        status: doc.status,
        #[cfg(feature = "oth")]
        il_odds: first_last(&lines),
        #[cfg(feature = "oth")]
        all_odds: lines,
        #[cfg(feature = "oth")]
        il_calcs: first_last(&calcs),
        #[cfg(feature = "oth")]
        all_calcs: calcs,
        home_lineup: doc.home_lineup.as_ref().map(|d| TeamLineup {
            formation: d.formation.clone(),
            coach: d.coach.clone(),
            starters: d.starters.iter().map(&lp).collect(),
            substitutes: d.substitutes.iter().map(&lp).collect(),
        }),
        away_lineup: doc.away_lineup.as_ref().map(|d| TeamLineup {
            formation: d.formation.clone(),
            coach: d.coach.clone(),
            starters: d.starters.iter().map(&lp).collect(),
            substitutes: d.substitutes.iter().map(&lp).collect(),
        }),
        events: doc
            .events
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|d| FootballEvent {
                        minute: d.minute,
                        extra: d.extra,
                        event_type: d.event_type.clone(),
                        player: d.player.clone(),
                        team: d.team.clone(),
                        assist: d.assist.clone(),
                        player_out: d.player_out.clone(),
                        note: d.note.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        stats: doc.stats.as_ref().map(|d| FootballStats {
            possession: crate::models::SideStats {
                home: d.possession.home,
                away: d.possession.away,
            },
            shots: crate::models::SideStatsInt {
                home: d.shots.home,
                away: d.shots.away,
            },
            shots_on_target: crate::models::SideStatsInt {
                home: d.shots_on_target.home,
                away: d.shots_on_target.away,
            },
            corners: crate::models::SideStatsInt {
                home: d.corners.home,
                away: d.corners.away,
            },
            fouls: crate::models::SideStatsInt {
                home: d.fouls.home,
                away: d.fouls.away,
            },
            offsides: crate::models::SideStatsInt {
                home: d.offsides.home,
                away: d.offsides.away,
            },
            yellow_cards: crate::models::SideStatsInt {
                home: d.yellow_cards.home,
                away: d.yellow_cards.away,
            },
            red_cards: crate::models::SideStatsInt {
                home: d.red_cards.home,
                away: d.red_cards.away,
            },
            passes: crate::models::SideStatsInt {
                home: d.passes.home,
                away: d.passes.away,
            },
            pass_accuracy: crate::models::SideStats {
                home: d.pass_accuracy.home,
                away: d.pass_accuracy.away,
            },
        }),
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
    }
}

async fn batch_enrich(docs: Vec<FootballDoc>) -> Result<Vec<Football>, String> {
    if docs.is_empty() {
        return Ok(vec![]);
    }
    let ids: Vec<&RecordId> = docs.iter().map(|d| &d.id).collect();
    let cat_ids: Vec<&RecordId> = docs.iter().map(|d| &d.category_id).collect();
    let topic_batch = topic_db::get_topics_batch(&ids).await?;
    let cat_map = category_db::get_categories_batch(&cat_ids).await?;
    let summary_map = analysis_db::get_summaries_batch(&ids).await?;
    let mut out = Vec::with_capacity(docs.len());
    for (i, doc) in docs.into_iter().enumerate() {
        let fid = rid_str(&doc.id);
        #[cfg(feature = "oth")]
        let lines: Vec<Line> = doc
            .lines
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|d| Line {
                        win: d.win,
                        draw: d.draw,
                        loss: d.loss,
                        created_at: common::ymdhmsz8(&d.created_at),
                    })
                    .collect()
            })
            .unwrap_or_default();
        #[cfg(feature = "oth")]
        let calcs: Vec<Calc> = doc
            .calcs
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|d| Calc {
                        s: d.s.clone(),
                        wdl: d.wdl.clone(),
                        tg: d.tg.clone(),
                        gd: d.gd.clone(),
                        created_at: common::ymdhmsz8(&d.created_at),
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(build_football(
            doc,
            #[cfg(feature = "oth")]
            lines,
            #[cfg(feature = "oth")]
            calcs,
            topic_batch.get(i).cloned().unwrap_or_default(),
            cat_map.get(&fid).cloned(),
            summary_map.get(&fid).cloned().flatten(),
            vec![],
        ));
    }
    Ok(out)
}

pub async fn get_home_footballs(limit: i64) -> Result<Vec<Football>, String> {
    let q = format!(
        "SELECT {FL} FROM footballs WHERE status >= 1 ORDER BY updated_at DESC LIMIT $limit"
    );
    let mut res = get_db()
        .query(&q)
        .bind(("limit", limit))
        .await
        .map_err(|e| e.to_string())?;
    batch_enrich(res.take(0).map_err(|e| e.to_string())?).await
}

pub async fn get_football_by_id(rid: &RecordId) -> Result<Option<Football>, String> {
    let doc: Option<FootballDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;
    match doc {
        Some(d) => {
            #[cfg(feature = "oth")]
            let lines: Vec<Line> = d
                .lines
                .as_ref()
                .map(|v| {
                    v.iter()
                        .map(|d| Line {
                            win: d.win,
                            draw: d.draw,
                            loss: d.loss,
                            created_at: common::ymdhmsz8(&d.created_at),
                        })
                        .collect()
                })
                .unwrap_or_default();
            #[cfg(feature = "oth")]
            let calcs: Vec<Calc> = d
                .calcs
                .as_ref()
                .map(|v| {
                    v.iter()
                        .map(|d| Calc {
                            s: d.s.clone(),
                            wdl: d.wdl.clone(),
                            tg: d.tg.clone(),
                            gd: d.gd.clone(),
                            created_at: common::ymdhmsz8(&d.created_at),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let topics = topic_db::get_topics_by_football_id(&d.id).await?;
            let category = category_db::get_category_by_id(&d.category_id).await?;
            let analyses = analysis_db::get_analyses_by_football_id(&d.id).await?;
            let summary = analyses
                .iter()
                .find(|a| a.status == 1)
                .and_then(|a| a.summary.clone())
                .filter(|s| !s.is_empty());
            Ok(Some(build_football(
                d,
                #[cfg(feature = "oth")]
                lines,
                #[cfg(feature = "oth")]
                calcs,
                topics,
                category,
                summary,
                analyses,
            )))
        }
        None => Ok(None),
    }
}

pub async fn get_random_football_id() -> Result<Option<String>, String> {
    let mut res = get_db().query("SELECT VALUE id FROM footballs WHERE status >= 1 AND kick_off_at >= time::now() - 1d ORDER BY rand() LIMIT 1").await.map_err(|e| e.to_string())?;
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
    let q = format!(
        "SELECT {FL} FROM footballs WHERE status >= $min AND status <= $max ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip"
    );
    let mut res = get_db()
        .query(&q)
        .bind(("min", status_min))
        .bind(("max", status_max))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let items = batch_enrich(docs).await?;
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
    let q = format!(
        "SELECT {FL} FROM footballs WHERE category_id = $cid AND status >= 1 ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip"
    );
    let mut res = get_db()
        .query(&q)
        .bind(("cid", category_rid.clone()))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let items = batch_enrich(docs).await?;
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
    let mut rel_res = get_db().query("SELECT VALUE football_id FROM topics_rel WHERE topic_id = $tid AND football_id IS NOT NONE").bind(("tid", topic_rid.clone())).await.map_err(|e| e.to_string())?;
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
        "SELECT {FL} FROM footballs WHERE id IN [{}] AND status >= 1 ORDER BY kick_off_at DESC",
        page_fids
            .iter()
            .map(|id| format!("{}", id))
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let items = batch_enrich(docs).await?;
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
    let q = format!("SELECT {FL} FROM footballs ORDER BY updated_at DESC LIMIT $ps START $skip");
    let mut res = get_db()
        .query(&q)
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let items = batch_enrich(docs).await?;
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

pub async fn insert_article(title: &str, category_id: &str, status: i8) -> Result<String, String> {
    let rid = common::into_rid(category_id, "categories");
    let mut res = get_db().query("CREATE footballs SET article_title = $title, category_id = $cid, ana_type = 0, status = $status, created_at = time::now(), updated_at = time::now()").bind(("title", title.to_string())).bind(("cid", rid)).bind(("status", status)).await.map_err(|e| e.to_string())?;
    let created: Option<RecordId> = res.take(0).map_err(|e| e.to_string())?;
    Ok(rid_str(&created.ok_or("create failed")?))
}
