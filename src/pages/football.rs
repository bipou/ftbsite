use crate::i18n::{t, t_display, use_i18n};
use crate::shared::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, CARD_SECTION, EMPTY, FLEX_WRAP_GAP, ITALIC, MAIN, NO_DATA,
    SECTION_H2, TEXT_MUTED, TEXT_WARN, TEXT_XS_MUTED,
};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::models::{Football, FootballEvent, FootballStats, TeamLineup};
use crate::shared::fns::get_username_by_id;
use crate::shared::locale::LocaleA;

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

#[component]
fn FootballHeader(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title_text = {
        let ht = f.home_team.clone();
        let at = f.away_team.clone();
        move || format!("{} vs {} – {}", ht, at, site_title!(i18n))
    };
    let loc = i18n.get_locale().to_string();
    let cat = Memo::new(move |_| f.category.as_ref().and_then(|c| c.name.get(&loc).cloned()));
    view! {
        <Title text=title_text/>
        <div class=CARD_SECTION>
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                        {f.home_team} <span class="text-gray-400 mx-2">"vs"</span> {f.away_team}
                    </h1>
                    <div class="text-sm text-gray-500 space-x-3">
                        <span>{move || t!(i18n, football_season)}{move || t!(i18n, colon)} {f.season}</span>
                        {move || if cat.get().is_some() {
                            Either::Left(view! { <span class=BADGE_GRAY>{cat.get().unwrap_or_default()}</span> })
                        } else {
                            Either::Right(())
                        }}
                    </div>
                </div>
                <div class="text-right text-sm text-gray-500">
                    <div>{move || t!(i18n, football_kick_off)}</div>
                    <div class="font-semibold text-blue-600">{f.kick_off_at_mdhm8}</div>
                    <div class=TEXT_XS_MUTED>"UTC: " {f.kick_off_at_mdhm}</div>
                </div>
            </div>
            <div class=format!("mt-3 {} flex gap-4 flex-wrap", TEXT_XS_MUTED)>
                <span>{move || t!(i18n, football_created)}{move || t!(i18n, colon)} {f.created_at}</span>
                <span>{move || t!(i18n, football_updated)}{move || t!(i18n, colon)} {f.updated_at}</span>
                <span>{move || t!(i18n, football_hits)}{move || t!(i18n, colon)} {f.hits}</span>
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
        return Either::Left(view! {
            <div class=CARD_SECTION>
                <p class=format!("text-gray-400 text-sm {}", ITALIC)>
                    {move || t!(i18n, not_calc)}
                </p>
            </div>
        });
    }
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, odds)}</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500 dark:text-gray-400">
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
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{o.created_at}</td>
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
        return Either::Left(view! {
            <div class=CARD_SECTION>
                <p class=format!("text-gray-400 text-sm {}", ITALIC)>
                    {move || t!(i18n, not_calc)}
                </p>
            </div>
        });
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
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{c.created_at}</td>
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
                {move || t!(i18n, football_formation)}{move || t!(i18n, colon)} {formation}
                {move || coach.as_ref().map(|c| view! {
                    <span class="ml-2">
                        {move || t!(i18n, football_coach)}{move || t!(i18n, colon)} {c.clone()}
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
                        Some(x) => format!("{}+{}'", e.minute, x),
                        None => format!("{}'", e.minute),
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
    let pos_label = t_display!(i18n, football_possession).to_string();
    let shots_label = t_display!(i18n, football_stat_shots).to_string();
    let sots_label = t_display!(i18n, football_stat_shots_on).to_string();
    let corn_label = t_display!(i18n, football_stat_corners).to_string();
    let foul_label = t_display!(i18n, football_stat_fouls).to_string();
    let offs_label = t_display!(i18n, football_stat_offsides).to_string();
    let yc_label = t_display!(i18n, football_stat_yellow_cards).to_string();
    let rc_label = t_display!(i18n, football_stat_red_cards).to_string();
    let pass_label = t_display!(i18n, football_stat_passes).to_string();
    let pacc_label = t_display!(i18n, football_stat_pass_accuracy).to_string();
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
                <div class="h-full bg-blue-500" style=format!("width:{}%", hl)></div>
                <div class="h-full bg-gray-400 dark:bg-gray-500" style=format!("width:{}%", 100 - hl)></div>
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
fn ArticleHeader(
    #[prop(into)] title: Option<String>,
    #[prop(into)] created: String,
    hits: u64,
) -> impl IntoView {
    let i18n = use_i18n();
    let title_text = title.unwrap_or_default();
    view! {
        <div class=CARD_SECTION>
            <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                {title_text}
            </h1>
            <div class=format!("mt-3 {} flex gap-4 flex-wrap", TEXT_XS_MUTED)>
                <span>{move || t!(i18n, football_created)}{move || t!(i18n, colon)} {created}</span>
                <span>{move || t!(i18n, football_hits)}{move || t!(i18n, colon)} {hits}</span>
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
    let is_ai = ana_type > 0;

    // 提取所需字段，避免 view! 中移动冲突
    let user_id = analysis.user_id.clone();
    let generated_at = analysis.generated_at;
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

    view! {
        <div class="mb-6">
            <div class=format!("{} mb-2", TEXT_XS_MUTED)>
                {move || {
                    if is_ai {
                        format!("{}", t_display!(i18n, analysis_ai))
                    } else {
                        author_name.get().flatten().unwrap_or_default()
                    }
                }}
                {" · "}
                {generated_at}
            </div>
            <div inner_html=content_html></div>
            {if is_ai {
                Either::Right(view! {
                    <p class="text-xs text-gray-400 mt-2">{format!("{}", t_display!(i18n, analysis_ai_label))}</p>
                })
            } else {
                Either::Left(())
            }}
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
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_analysis)}</h2>
            <div class="prose prose-sm dark:prose-invert max-w-none">
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
    view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_result)}</h2>
            {match (&s, &wdl, &tg, &gd) {
                (Some(s), Some(wdl), Some(tg), Some(gd)) => {
                    let s = s.clone();
                    let wdl = wdl.clone();
                    let tg = *tg;
                    let gd = *gd;
                    Either::Right(view! {
                        <table class="w-full text-sm text-left">
                            <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500 dark:text-gray-400">
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
                    })
                }
                _ => Either::Left(view! {
                    <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_full)}</p>
                }),
            }}
        </div>
    }
}

#[component]
fn DetailTopicsSection(topics: Vec<crate::models::Topic>) -> impl IntoView {
    let i18n = use_i18n();
    if topics.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-4 mb-6">
                <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, football_keys_tags)}{move || t!(i18n, colon)}</p>
                <div class=FLEX_WRAP_GAP>
                    {topics.iter().map(|t| {
                        let kid = crate::shared::common::record_key(&t.id).to_string();
                        view! {
                            <a href=format!("/footballs?topic={}", kid) class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        })
    }
}

#[component]
fn FootballDetail(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let ana_type = f.ana_type;
    let header_f = f.clone();
    let extra = render_detail_extra(&f);
    let home_lineup = f.home_lineup;
    let away_lineup = f.away_lineup;
    let events = f.events;
    let stats = f.stats;
    let analyses = f.analyses;
    let topics = f.topics;
    let result_s = f.result_s;
    let result_wdl = f.result_wdl;
    let result_tg = f.result_tg;
    let result_gd = f.result_gd;
    let article_title = f.article_title;
    view! {
        <p class={format!("{} text-center mb-4", TEXT_WARN)}>
            {move || t!(i18n, site_warn)}
        </p>
        {if ana_type == 0 {
            Either::Left(view! {
                <ArticleHeader title=article_title created=f.created_at hits=f.hits/>
            })
        } else {
            Either::Right(view! {
                <div>
                    <FootballHeader f=header_f/>
                    <ResultDetail s=result_s wdl=result_wdl tg=result_tg gd=result_gd/>
                    <LineupSection home=home_lineup away=away_lineup/>
                    <EventsSection events=events/>
                    <StatsSection stats=stats/>
                    {extra}
                </div>
            })
        }}
        <AnalysisSection analyses=analyses ana_type=ana_type/>
        <DetailTopicsSection topics=topics/>
    }
}

// ── Three-way view type alias ───────────────────────────────────────────
type DetailResult<A, B, C> = Either<A, Either<B, C>>;

#[component]
pub fn FootballDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let data = Resource::new_blocking(
        move || id(),
        |id| async move { get_football_and_increment(id).await },
    );

    view! {
        <Nav/>
        <main class=MAIN>
            <Suspense fallback=move || view! {
                <div class=format!("{} text-gray-400", EMPTY)>
                    {move || t!(i18n, loading)}
                </div>
            }>
                {move || data.get().map(|result| match result {
                    Err(e) => DetailResult::Left(view! {
                        <p class="text-red-500 text-center py-8">{e.to_string()}</p>
                    }),
                    Ok(None) => DetailResult::Right(Either::Left(view! {
                        <div class=EMPTY>
                            <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                            <LocaleA href="/footballs" class="btn-primary">{move || t!(i18n, go_list)}</LocaleA>
                        </div>
                    })),
                    Ok(Some(f)) => DetailResult::Right(Either::Right(view! { <FootballDetail f=f/> })),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}

/// /rand → 302 跳转到随机球赛详情
#[component]
pub fn RandomRedirect() -> impl IntoView {
    let params = use_params_map();
    let locale = params.read().get("locale").unwrap_or_default();

    let redirect = Resource::new_blocking(
        move || locale.clone(),
        |locale| async move { redirect_to_random_football(locale).await },
    );

    view! {
        <Suspense fallback=|| view! { <div></div> }>
            {move || { let _ = redirect.get(); view! { <div></div> } }}
        </Suspense>
    }
}

#[server]
async fn redirect_to_random_football(locale: String) -> Result<(), ServerFnError> {
    use crate::server::football_db;
    use crate::shared::common::record_key;
    use axum::http::{HeaderValue, StatusCode, header};
    use leptos_axum::ResponseOptions;

    let url = match football_db::get_random_football_id().await {
        Ok(Some(full_id)) => {
            let kid = record_key(&full_id);
            format!("/{}/footballs/{}", locale, kid)
        }
        _ => format!("/{}/footballs", locale),
    };

    let resp = expect_context::<ResponseOptions>();
    resp.set_status(StatusCode::FOUND);
    resp.insert_header(
        header::LOCATION,
        HeaderValue::from_str(&url).map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(())
}
