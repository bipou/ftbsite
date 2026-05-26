use crate::i18n::{t, use_i18n};
use crate::models::Football;
use crate::shared::common::Either3;
use crate::shared::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, BADGE_GRAY_NO_UL, FLEX_BETWEEN, HOVER_SHADOW, ITALIC,
    TEXT_SUBTLE, TEXT_XS_MUTED,
};
#[cfg(feature = "oth")]
use crate::shared::constant::{BADGE_GREEN, BADGE_RED, TEXT_MUTED};
use crate::shared::locale::{LocaleA, use_locale};
use leptos::either::Either;
use leptos::prelude::*;

fn status_class(status: i8) -> &'static str {
    match status {
        4 => "fc-status-4",
        3 => "fc-status-3",
        2 => "fc-status-2",
        1 => "fc-status-1",
        _ => "fc-status-0",
    }
}

fn status_badge(status: i8) -> &'static str {
    match status {
        4 => "⭐🔥",
        3 => "⭐ Pick",
        2 => "🔥 Hot",
        1 => "Published",
        0 => "Draft",
        _ => "—",
    }
}

#[component]
fn CatBadge(
    #[prop(into)] name: Signal<Option<String>>,
    #[prop(into)] kid: Option<String>,
) -> impl IntoView {
    let loc_str = use_locale();
    move || {
        let n = name.get();
        if n.is_none() {
            Either3::Left(())
        } else if let Some(kid) = &kid {
            let href = format!("/{}/footballs?category={}", loc_str.get(), kid);
            Either3::Right(Either::Left(view! {
                <a href=href class=BADGE_GRAY_NO_UL>{n.unwrap_or_default()}</a>
            }))
        } else {
            Either3::Right(Either::Right(
                view! { <span class=BADGE_GRAY>{n.unwrap_or_default()}</span> },
            ))
        }
    }
}

#[component]
fn ResultSection(
    #[prop(into)] s: Option<String>,
    #[prop(into)] wdl: Option<u8>,
    #[prop(into)] tg: Option<u8>,
) -> impl IntoView {
    let i18n = use_i18n();
    match (s, wdl, tg) {
        (Some(s), Some(wdl), Some(tg)) => Either::Right(view! {
            <div class="text-xs flex items-center gap-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_result)}</span>
                <span class="font-semibold text-blue-700 dark:text-blue-300">
                    <span class="mr-4">{move || t!(i18n, football_s)} ": " {s}</span>
                    <span class="mr-4">{move || t!(i18n, football_wdl)} ": " {wdl}</span>
                    <span>{move || t!(i18n, football_tg)} ": " {tg}</span>
                </span>
            </div>
        }),
        _ => Either::Left(view! {
            <p class=format!("{} {}", TEXT_XS_MUTED, ITALIC)>{move || t!(i18n, not_finished)}</p>
        }),
    }
}

#[allow(unused_variables)]
fn render_card_extra(football: &Football) -> impl IntoView + use<> {
    #[cfg(feature = "oth")]
    {
        let odds = football.il_odds.clone();
        let calcs = football.il_calcs.clone();
        view! {
            <OddsSection odds=odds/>
            <CalcsSection calcs=calcs/>
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
fn OddsSection(odds: Vec<crate::models::Line>) -> impl IntoView {
    let i18n = use_i18n();
    if odds.is_empty() {
        return Either::Left(view! {
            <p class=format!("{} {} mb-2", TEXT_XS_MUTED, ITALIC)>
                {move || t!(i18n, not_calc)}
            </p>
        });
    }
    let init = odds.first().cloned();
    let last = odds.last().cloned();
    Either::Right(view! {
        <div class="text-xs space-y-1 mb-2">
            {init.map(|o| view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_init_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {format!("{:.2}", o.win)}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {format!("{:.2}", o.draw)}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {format!("{:.2}", o.loss)}</span>
                </div>
            })}
            {last.and_then(|o| if odds.len() > 1 { Some(view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_last_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {format!("{:.2}", o.win)}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {format!("{:.2}", o.draw)}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {format!("{:.2}", o.loss)}</span>
                </div>
            }) } else { None })}
        </div>
    })
}

#[cfg(feature = "oth")]
#[component]
fn CalcsSection(calcs: Vec<crate::models::Calc>) -> impl IntoView {
    let i18n = use_i18n();
    if calcs.is_empty() {
        return Either::Left(());
    }
    let init = calcs.first().cloned();
    let last = calcs.last().cloned();
    Either::Right(view! {
        <div class="text-xs space-y-1 mb-2 border-t border-gray-100 dark:border-gray-700 pt-2">
            {init.map(|c| view! {
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_init_calc)}</span>
                    <span class=TEXT_MUTED>
                        {move || t!(i18n, football_s)} ": " {c.s}
                        " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                        " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                        " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                    </span>
                </div>
            })}
            {last.and_then(|c| if calcs.len() > 1 { Some(view! {
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_last_calc)}</span>
                    <span class=TEXT_MUTED>
                        {move || t!(i18n, football_s)} ": " {c.s}
                        " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                        " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                        " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                    </span>
                </div>
            }) } else { None })}
        </div>
    })
}

#[component]
pub fn FootballCard(football: Football) -> impl IntoView {
    let i18n = use_i18n();
    let extra = render_card_extra(&football);
    let card_class = format!(
        "card p-4 {} {}",
        HOVER_SHADOW,
        status_class(football.status)
    );
    let title = football.title();
    let season = football.season;
    let kick_off = football.kick_off_at_mdhm8;
    let status = football.status;
    let hits = football.hits;
    let topics = football.topics;
    let summary = football.summary;
    let summary_view = summary.map(|s| {
        view! {
            <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mt-2">{s}</p>
        }
    });
    let detail_path = format!(
        "/footballs/{}",
        crate::shared::common::record_key(&football.id)
    );
    let category = football.category;
    let cat_kid = category
        .as_ref()
        .map(|c| crate::shared::common::record_key(&c.id).to_string());
    let cat_name = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category.as_ref().and_then(|c| c.name.get(&loc).cloned())
    });

    view! {
        <div class=card_class>
            <div class=format!("{} mb-2", FLEX_BETWEEN)>
                <LocaleA href=detail_path target="_blank" rel="noopener noreferrer" class="font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 no-underline text-lg leading-tight truncate">
                    {title}
                </LocaleA>
                <span class="text-sm text-gray-400 ml-2 whitespace-nowrap">{status_badge(status)}</span>
            </div>

            <div class=format!("text-sm {} mb-3 space-x-2", TEXT_SUBTLE)>
                <span>{season}</span>
                <CatBadge name=cat_name kid=cat_kid/>
                <span class="text-blue-500">{kick_off}</span>
            </div>

            {extra}
            <ResultSection s=football.result_s wdl=football.result_wdl tg=football.result_tg/>
            {summary_view}

            <div class=format!("{} mt-3", FLEX_BETWEEN)>
                <div class="flex flex-wrap gap-1">
                    {topics.into_iter().map(|topic| {
                        let kid = crate::shared::common::record_key(&topic.id).to_string();
                        let href = format!("/footballs?topic={}", kid);
                        let name = topic.name;
                        view! {
                            <LocaleA href=href class=format!("text-sm {}", BADGE_BLUE_NO_UL)>{name}</LocaleA>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <span class="text-sm text-gray-400">{move || t!(i18n, football_hits)} {hits}</span>
            </div>
        </div>
    }
}
