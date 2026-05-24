use crate::i18n::{t, use_i18n};
use crate::shared::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, CARD_SECTION, EMPTY, FLEX_WRAP_GAP, ITALIC, MAIN, NO_DATA,
    SECTION_H2, TEXT_XS_MUTED,
};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::models::Football;
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
fn MatchHeader(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title_text = format!("{} vs {} – {}", f.home_team, f.away_team, site_title!(i18n));
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
                        <span>{move || t!(i18n, football_season)} " " {f.season}</span>
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
                <span>{move || t!(i18n, football_created)} ": " {f.created_at}</span>
                <span>{move || t!(i18n, football_updated)} ": " {f.updated_at}</span>
                <span>{move || t!(i18n, football_hits)} {f.hits}</span>
            </div>
        </div>
    }
}

#[component]
fn OddsTable(odds: Vec<crate::models::FootballLine>) -> impl IntoView {
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

#[component]
fn CalcsTable(calcs: Vec<crate::models::FootballOver>) -> impl IntoView {
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

#[component]
fn OverDetail(
    #[prop(into)] s: Option<String>,
    #[prop(into)] wdl: Option<u8>,
    #[prop(into)] tg: Option<u8>,
    #[prop(into)] gd: Option<i8>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_over)}</h2>
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
                <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, football_keys_tags)}</p>
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
    let header_f = f.clone();
    let mut odds = f.all_odds;
    odds.reverse();
    let mut calcs = f.all_calc_over;
    calcs.reverse();
    let topics = f.topics;
    let result_s = f.result_s;
    let result_wdl = f.result_wdl;
    let result_tg = f.result_tg;
    let result_gd = f.result_gd;
    view! {
        <MatchHeader f=header_f/>
        <OverDetail s=result_s wdl=result_wdl tg=result_tg gd=result_gd/>
        <CalcsTable calcs=calcs/>
        <OddsTable odds=odds/>
        <DetailTopicsSection topics=topics/>
        <p class="text-xs text-red-400 text-center mt-4">{move || t!(i18n, site_warn)}</p>
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
