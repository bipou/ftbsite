use crate::i18n::{t, t_display, use_i18n};
use crate::shared::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, BADGE_GRAY_NO_UL, CARD_SECTION, FLEX_WRAP_GAP, SECTION_H2,
    TEXT_MUTED, TEXT_WARN, TEXT_XS_MUTED,
};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::models::{Football, FootballEvent, FootballStats, TeamLineup};
use crate::shared::fns::get_username_by_id;
use crate::shared::locale::use_locale;

#[server]
pub async fn get_football_and_increment(id: String) -> Result<Option<Football>, ServerFnError> {
    use crate::server::football_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&id, "footballs");
    let _ = football_db::increment_hits(&rid).await;
    football_db::get_football_by_id(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_analyses(
    id: String,
) -> Result<Vec<crate::models::FootballAnalysis>, ServerFnError> {
    use crate::server::analysis_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&id, "footballs");
    analysis_db::get_analyses_by_football_id(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
fn FootballHeader(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let base = f.title();
    let base2 = base.clone();
    let title_text = move || [&base2, " \u{2013} ", &site_title!(i18n)].join("");
    let category_name = f.category_name;
    let cat_kid = category_name
        .as_ref()
        .map(|_| crate::shared::common::record_key(&f.category_id).to_string());
    let cat = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category_name.as_ref().and_then(|c| c.get(&loc).cloned())
    });
    let season = f.season;
    let article_title = f.article_title;
    let kick_off_mdhm = f.kick_off_at_mdhm;
    let kick_off_mdhm8 = f.kick_off_at_mdhm8;
    view! {
        <Title text=title_text/>
        <div class=CARD_SECTION>
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                        {base}
                    </h1>
                    <div class="text-sm text-gray-500 space-x-3">
                        {match &season {
                            Some(s) => Either::Left(view! {
                                <span>{move || t!(i18n, football_season)}{s.clone()}</span>
                            }),
                            None => Either::Right(()),
                        }}
                        {move || {
                            let n = cat.get();
                            if n.is_none() {
                                Either::Left(())
                            } else if let Some(kid) = &cat_kid {
                                let href = ["/", &loc_str.get(), "/footballs?category=", kid].join("");
                                Either::Right(Either::Left(view! {
                                    <a href=href class=BADGE_GRAY_NO_UL>{n.unwrap_or_default()}</a>
                                }))
                            } else {
                                Either::Right(Either::Right(
                                    view! { <span class=BADGE_GRAY>{n.unwrap_or_default()}</span> },
                                ))
                            }
                        }}
                        {match &article_title {
                            Some(at) => Either::Left(view! { <span class=BADGE_GRAY>{at.clone()}</span> }),
                            None => Either::Right(()),
                        }}
                    </div>
                </div>
                {match (kick_off_mdhm8.as_ref(), kick_off_mdhm.as_ref()) {
                    (Some(mdhm8), Some(mdhm)) => Either::Left(view! {
                        <div class="text-right text-sm text-gray-500">
                            <div>{move || t!(i18n, football_kick_off)}</div>
                            <div class="font-semibold text-blue-600">{mdhm8.clone()}</div>
                            <div class=TEXT_XS_MUTED>"UTC: " {mdhm.clone()}</div>
                        </div>
                    }),
                    _ => Either::Right(()),
                }}
            </div>
            <div class=["mt-3", TEXT_XS_MUTED, "flex", "gap-4", "flex-wrap"].join(" ")>
                <span>{move || t!(i18n, football_created)} ": " {f.created_at}</span>
                <span>{move || t!(i18n, football_updated)} ": " {f.updated_at}</span>
                <span>{move || t!(i18n, football_hits)}{f.hits}</span>
            </div>
        </div>
    }
}

#[allow(unused_variables)]
fn render_detail_extra(f: &Football) -> impl IntoView + use<> {
    #[cfg(feature = "oth")]
    {
        let mut odds = f.all_odds.clone();
        odds.reverse();
        let mut calcs = f.all_calcs.clone();
        calcs.reverse();
        view! {
            <CalcsTable calcs=calcs/>
            <OddsTable odds=odds/>
        }
        .into_view()
    }
    #[cfg(not(feature = "oth"))]
    {
        ().into_view()
    }
}

// ── oth-only components ─────────────────────────────────────────────────────

#[cfg(feature = "oth")]
#[component]
fn OddsTable(odds: Vec<crate::models::Line>) -> impl IntoView {
    let i18n = use_i18n();
    if odds.is_empty() {
        return Either::Left(());
    }
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, odds)}</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-800 text-xs text-gray-500 dark:text-gray-300">
                        <tr>
                            <th class="px-4 py-2">{move || t!(i18n, football_win)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_draw)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_loss)}</th>
                            <th class="px-4 py-2"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {odds.into_iter().map(|o| view! {
                            <tr class="table-row">
                                <td class="px-4 py-2">{format!("{:.2}", o.win)}</td>
                                <td class="px-4 py-2">{format!("{:.2}", o.draw)}</td>
                                <td class="px-4 py-2">{format!("{:.2}", o.loss)}</td>
                                <td class=["px-4", "py-2", TEXT_XS_MUTED].join(" ")>{o.created_at}</td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    })
}

#[cfg(feature = "oth")]
#[component]
fn CalcsTable(calcs: Vec<crate::models::Calc>) -> impl IntoView {
    let i18n = use_i18n();
    if calcs.is_empty() {
        return Either::Left(());
    }
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, calc)}</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500">
                        <tr>
                            <th class="px-4 py-2">{move || t!(i18n, football_s)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_wdl)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_tg)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_gd)}</th>
                            <th class="px-4 py-2"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {calcs.into_iter().map(|c| view! {
                            <tr class="table-row">
                                <td class="px-4 py-2">{c.s}</td>
                                <td class="px-4 py-2">{c.wdl}</td>
                                <td class="px-4 py-2">{c.tg}</td>
                                <td class="px-4 py-2">{c.gd}</td>
                                <td class=["px-4", "py-2", TEXT_XS_MUTED].join(" ")>{c.created_at}</td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    })
}

// ── 阵容 ─────────────────────────────────────────────────────────────────

fn event_icon(t: &str) -> String {
    match t {
        "goal" | "penalty_goal" => "⚽",
        "own_goal" => "🅾",
        "yellow_card" => "🟨",
        "red_card" => "🟥",
        "sub" => "🔄",
        _ => "•",
    }
    .into()
}

#[component]
fn LineupSection(home: Option<TeamLineup>, away: Option<TeamLineup>) -> impl IntoView {
    let i18n = use_i18n();
    if home.is_none() && away.is_none() {
        return Either::Left(());
    }
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_lineups)}</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                {home.map(|l| view! { <TeamView team=l/> })}
                {away.map(|l| view! { <TeamView team=l/> })}
            </div>
        </div>
    })
}

#[component]
fn TeamView(team: TeamLineup) -> impl IntoView {
    let i18n = use_i18n();
    let formation = team.formation;
    let coach = team.coach;
    let starters = team.starters;
    let subs = team.substitutes;
    view! {
        <div>
            <div class="text-sm text-gray-500 mb-1">
                {move || t!(i18n, football_formation)} ": " {formation}
                {move || coach.as_ref().map(|c| view! {
                    <span class="ml-2">
                        {move || t!(i18n, football_coach)} ": " {c.clone()}
                    </span>
                })}
            </div>
            <div class="text-xs space-y-0.5">
                {starters.into_iter().map(|p| view! {
                    <div class="flex gap-2">
                        <span class="w-6 text-right text-gray-400">{p.number}</span>
                        <span>{p.name}</span>
                        <span class="text-gray-400">{p.position}</span>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
            {if !subs.is_empty() {
                Either::Left(view! {
                    <p class="text-xs text-gray-400 mt-2 mb-1">{move || t!(i18n, football_substitutes)}</p>
                    <div class="text-xs space-y-0.5">
                        {subs.into_iter().map(|p| view! {
                            <div class="flex gap-2">
                                <span class="w-6 text-right text-gray-400">{p.number}</span>
                                <span>{p.name}</span>
                                <span class="text-gray-400">{p.position}</span>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                })
            } else {
                Either::Right(())
            }}
        </div>
    }
}

// ── 赛况时间线 ───────────────────────────────────────────────────────────

#[component]
fn EventsSection(events: Vec<FootballEvent>) -> impl IntoView {
    let i18n = use_i18n();
    if events.is_empty() {
        return Either::Left(());
    }
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_events)}</h2>
            <div class="space-y-2">
                {events.into_iter().map(|e| {
                    let icon = event_icon(&e.event_type);
                    let minute_str = match e.extra {
                                            Some(x) => [&e.minute.to_string(), "+", &x.to_string(), "'"].join(""),
                                            None => [&e.minute.to_string(), "'"].join(""),
                    };
                    let note = e.note;
                    view! {
                        <div class="flex items-start gap-3 text-sm">
                            <span class="w-10 text-right text-gray-400 shrink-0">{minute_str}</span>
                            <span class="w-5 text-center">{icon}</span>
                            <span class=TEXT_MUTED>{note}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    })
}

// ── 技术统计 ─────────────────────────────────────────────────────────────

#[component]
fn StatsSection(stats: Option<FootballStats>) -> impl IntoView {
    let i18n = use_i18n();
    let s = match stats {
        Some(s) => s,
        None => return Either::Left(()),
    };
    let pos_label = Signal::derive(move || t_display!(i18n, football_possession).to_string());
    let shots_label = Signal::derive(move || t_display!(i18n, football_stat_shots).to_string());
    let sots_label = Signal::derive(move || t_display!(i18n, football_stat_shots_on).to_string());
    let corn_label = Signal::derive(move || t_display!(i18n, football_stat_corners).to_string());
    let foul_label = Signal::derive(move || t_display!(i18n, football_stat_fouls).to_string());
    let offs_label = Signal::derive(move || t_display!(i18n, football_stat_offsides).to_string());
    let yc_label = Signal::derive(move || t_display!(i18n, football_stat_yellow_cards).to_string());
    let rc_label = Signal::derive(move || t_display!(i18n, football_stat_red_cards).to_string());
    let pass_label = Signal::derive(move || t_display!(i18n, football_stat_passes).to_string());
    let pacc_label =
        Signal::derive(move || t_display!(i18n, football_stat_pass_accuracy).to_string());
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_stats)}</h2>
            <div class="space-y-2 text-sm">
                <StatRow label=pos_label hv=s.possession.home av=s.possession.away pct=true/>
                <StatRowInt label=shots_label hv=s.shots.home av=s.shots.away/>
                <StatRowInt label=sots_label hv=s.shots_on_target.home av=s.shots_on_target.away/>
                <StatRowInt label=corn_label hv=s.corners.home av=s.corners.away/>
                <StatRowInt label=foul_label hv=s.fouls.home av=s.fouls.away/>
                <StatRowInt label=offs_label hv=s.offsides.home av=s.offsides.away/>
                <StatRowInt label=yc_label hv=s.yellow_cards.home av=s.yellow_cards.away/>
                <StatRowInt label=rc_label hv=s.red_cards.home av=s.red_cards.away/>
                <StatRowInt label=pass_label hv=s.passes.home av=s.passes.away/>
                <StatRow label=pacc_label hv=s.pass_accuracy.home av=s.pass_accuracy.away pct=true/>
            </div>
        </div>
    })
}

#[component]
fn StatRow(
    #[prop(into)] label: Signal<String>,
    hv: f32,
    av: f32,
    #[prop(default = false)] pct: bool,
) -> impl IntoView {
    let hv_d = if pct {
        format!("{hv:.1}%")
    } else {
        hv.to_string()
    };
    let av_d = if pct {
        format!("{av:.1}%")
    } else {
        av.to_string()
    };
    let total = hv + av;
    let hl = if total > 0.0 {
        (hv / total * 100.0) as u8
    } else {
        50
    };
    view! {
        <div>
            <div class="flex justify-between text-xs text-gray-500 mb-0.5">
                <span>{hv_d}</span>
                <span>{label}</span>
                <span>{av_d}</span>
            </div>
            <div class="h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden flex">
                <div class="h-full bg-blue-500" style=["width:", &hl.to_string(), "%"].join("")></div>
                                <div class="h-full bg-gray-400 dark:bg-gray-500" style=["width:", &(100 - hl).to_string(), "%"].join("")></div>
            </div>
        </div>
    }
}

#[component]
fn StatRowInt(#[prop(into)] label: Signal<String>, hv: u16, av: u16) -> impl IntoView {
    view! {
        <StatRow label=label hv=hv as f32 av=av as f32 pct=false/>
    }
}

// ── 分析文章 ────────────────────────────────────────────────────────────

#[component]
fn ArticleHeader(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title_text = f.title();
    let page_title_text = title_text.clone();
    let loc_str = use_locale();
    let category_name = f.category_name;
    let season = f.season;
    let cat_kid = category_name
        .as_ref()
        .map(|_| crate::shared::common::record_key(&f.category_id).to_string());
    let cat = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category_name.as_ref().and_then(|c| c.get(&loc).cloned())
    });
    view! {
        <Title text=move || [&page_title_text, " – ", &site_title!(i18n)].join("")/>
        <div class=CARD_SECTION>
            <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                {title_text}
            </h1>
            <div class="text-sm text-gray-500 space-x-3 mb-3">
                {match &season {
                    Some(s) => Either::Left(view! {
                        <span>{move || t!(i18n, football_season)}{s.clone()}</span>
                    }),
                    None => Either::Right(()),
                }}
                {move || {
                    let n = cat.get();
                    if n.is_none() {
                        Either::Left(())
                    } else if let Some(kid) = &cat_kid {
                        let href = ["/", &loc_str.get(), "/footballs?category=", kid].join("");
                        Either::Right(Either::Left(view! {
                            <a href=href class=BADGE_GRAY_NO_UL>{n.unwrap_or_default()}</a>
                        }))
                    } else {
                        Either::Right(Either::Right(
                            view! { <span class=BADGE_GRAY>{n.unwrap_or_default()}</span> },
                        ))
                    }
                }}
            </div>
            <div class=["mt-3", TEXT_XS_MUTED, "flex", "gap-4", "flex-wrap"].join(" ")>
                <span>{move || t!(i18n, football_created)} ": " {f.created_at}</span>
                <span>{move || t!(i18n, football_updated)} ": " {f.updated_at}</span>
                <span>{move || t!(i18n, football_hits)}{f.hits}</span>
            </div>
        </div>
    }
}

#[component]
fn AnalysisCard(
    analysis: crate::models::FootballAnalysis,
    #[prop(into)] ana_type: u8,
) -> impl IntoView {
    let i18n = use_i18n();

    // 提取所需字段，避免 view! 中移动冲突
    let user_id = analysis.user_id;
    let content_html = analysis.content_html;

    let author_name = Resource::new(
        move || user_id.clone(),
        |uid| async move {
            match uid {
                Some(id) if !id.is_empty() => get_username_by_id(id).await.ok().flatten(),
                _ => None,
            }
        },
    );

    // ana_type → i18n key
    let label = match ana_type {
        0 => t_display!(i18n, user_analysis).to_string(),
        1 => t_display!(i18n, pre_match_analysis).to_string(),
        _ => t_display!(i18n, post_match_review).to_string(),
    };

    view! {
        <div class="mb-6">
            <div class=[TEXT_XS_MUTED, "mb-2"].join(" ")>
                {match ana_type {
                    0 => Either::Left(view! {
                        <Suspense fallback=|| "">
                            {move || author_name.get().flatten().unwrap_or_default()}
                        </Suspense>
                    }),
                    _ => Either::Right(view! { <span>{label}</span> }),
                }}
            </div>
            <div inner_html=content_html></div>
        </div>
    }
}

#[component]
fn AnalysisSection(
    analyses: Vec<crate::models::FootballAnalysis>,
    #[prop(into)] ana_type: u8,
) -> impl IntoView {
    let i18n = use_i18n();
    if analyses.is_empty() {
        return Either::Left(());
    }
    let title = match ana_type {
        0 => t_display!(i18n, user_analysis).to_string(),
        1 => t_display!(i18n, pre_match_analysis).to_string(),
        _ => t_display!(i18n, post_match_review).to_string(),
    };
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{title}</h2>
            <div class="prose prose-sm max-w-none">
                {analyses.into_iter().map(|a| view! {
                    <AnalysisCard analysis=a ana_type=ana_type/>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    })
}

#[component]
fn ResultDetail(
    #[prop(into)] s: Option<String>,
    #[prop(into)] wdl: Option<u8>,
    #[prop(into)] tg: Option<u8>,
    #[prop(into)] gd: Option<i8>,
) -> impl IntoView {
    let i18n = use_i18n();
    match (&s, &wdl, &tg, &gd) {
        (Some(s), Some(wdl), Some(tg), Some(gd)) => {
            let s = s.clone();
            let wdl = *wdl;
            let tg = *tg;
            let gd = *gd;
            Either::Right(view! {
                <div class=CARD_SECTION>
                    <h2 class=SECTION_H2>{move || t!(i18n, football_result)}</h2>
                    <table class="w-full text-sm text-left">
                        <thead class="bg-gray-50 dark:bg-gray-800 text-xs text-gray-500 dark:text-gray-300">
                            <tr>
                                <th class="px-4 py-2 text-center">{move || t!(i18n, football_s)}</th>
                                <th class="px-4 py-2 text-center">{move || t!(i18n, football_wdl)}</th>
                                <th class="px-4 py-2 text-center">{move || t!(i18n, football_tg)}</th>
                                <th class="px-4 py-2 text-center">{move || t!(i18n, football_gd)}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr class="table-row">
                                <td class="px-4 py-2 font-semibold text-blue-600 dark:text-blue-400 text-center">{s}</td>
                                <td class="px-4 py-2 font-semibold text-blue-600 dark:text-blue-400 text-center">{wdl}</td>
                                <td class="px-4 py-2 font-semibold text-blue-600 dark:text-blue-400 text-center">{tg}</td>
                                <td class="px-4 py-2 font-semibold text-blue-600 dark:text-blue-400 text-center">{gd}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            })
        }
        _ => Either::Left(()),
    }
}

#[component]
fn DetailTopicsSection(topics: Vec<crate::models::Topic>) -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    match topics.is_empty() {
        true => Either::Left(()),
        false => Either::Right(view! {
            <div class="card p-4 mb-6">
                <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, football_keys_tags)}</p>
                <div class=FLEX_WRAP_GAP>
                    {topics.iter().map(|t| {
                        let kid = crate::shared::common::record_key(&t.id).to_string();
                        let href = ["/", &loc_str.get(), "/footballs?topic=", &kid].join("");
                        view! {
                            <a href=href class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        }),
    }
}

#[component]
pub fn FootballDetail(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let ana_type = f.ana_type;
    let header_f = f.clone();
    let extra = render_detail_extra(&f);
    let home_lineup = f.home_lineup;
    let away_lineup = f.away_lineup;
    let events = f.events;
    let stats = f.stats;
    let topics = f.topics;
    let result_s = f.result_s;
    let result_wdl = f.result_wdl;
    let result_tg = f.result_tg;
    let result_gd = f.result_gd;
    let football_id = f.id.clone();

    let analyses_res = Resource::new(move || football_id.clone(), |fid| get_analyses(fid));
    view! {
        <p class={[TEXT_WARN, "text-center", "mb-4"].join(" ")}>
            {move || t!(i18n, site_warn)}
        </p>
        {match ana_type == 0 {
            true => Either::Left(view! {
                <ArticleHeader f=header_f/>
            }),
            false => Either::Right(view! {
                <div>
                    <FootballHeader f=header_f/>
                    <ResultDetail s=result_s wdl=result_wdl tg=result_tg gd=result_gd/>
                    <LineupSection home=home_lineup away=away_lineup/>
                    <EventsSection events=events/>
                    <StatsSection stats=stats/>
                    {extra}
                </div>
            }),
        }}
        <Suspense fallback=|| ()>
            {move || analyses_res.get().and_then(|r| r.ok()).map(|analyses| {
                view! {
                    {analyses.first().and_then(|a| a.summary.clone()).filter(|s| !s.is_empty()).map(|s| view! {
                        <div class=CARD_SECTION>
                            <p class="text-sm text-gray-600 dark:text-gray-300">{s}</p>
                        </div>
                    })}
                    <AnalysisSection analyses=analyses ana_type=ana_type/>
                }
            })}
        </Suspense>
        <DetailTopicsSection topics=topics/>
    }
}
