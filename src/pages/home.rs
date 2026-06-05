use crate::detail_close_nav;
use crate::detail_open_nav;
use crate::i18n::{t, t_display, use_i18n};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use serde::{Deserialize, Serialize};

use crate::components::{ArticleCard, FootballCard, SlidePanel};
use crate::models::Football;
use crate::pages::football::{FootballDetail, get_football_and_increment};

use crate::shared::common::Either3;
use crate::shared::constant::{
    EMPTY, GRID_3, NO_DATA, SLIDE_SIZED_LG, TEXT_SUBTLE, TEXT_WARN, WIDE,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeData {
    pub user: Vec<Football>,
    pub pre: Vec<Football>,
    pub post: Vec<Football>,
}

#[server]
pub async fn get_home_data() -> Result<HomeData, ServerFnError> {
    use crate::server::football_db;
    let (user, pre, post) = football_db::get_home_footballs(9)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(HomeData { user, pre, post })
}

#[component]
fn HomeSection(
    ana_type: u8,
    footballs: Vec<Football>,
    on_click: Callback<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section class="mb-12">
            <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-200 border-b border-blue-200 dark:border-blue-800 pb-2 mb-4 flex items-center gap-2">
                <span class="text-blue-500">{match ana_type { 0 => "✍️", 1 => "📊", _ => "📋" }}</span>
                {move || match ana_type {
                    0 => { t_display!(i18n, user_analysis).to_string() },
                    1 => { t_display!(i18n, pre_match_analysis).to_string() },
                    _ => { t_display!(i18n, post_match_review).to_string() },
                }}
            </h2>
            {match footballs.is_empty() {
                true => Either::Left(view! {
                    <div class=EMPTY>
                        <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                    </div>
                }),
                false => Either::Right(view! {
                    <div class={GRID_3}>
                        {footballs.into_iter().map(|f| {
                            match f.ana_type == 0 {
                                true => Either::Left(view! { <ArticleCard football=f on_click=on_click/> }),
                                false => Either::Right(view! { <FootballCard football=f on_click=on_click/> }),
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }),
            }}
        </section>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();
    let data = Resource::new_blocking(|| (), |_| get_home_data());

    let selected_id: RwSignal<Option<String>> = RwSignal::new(None);
    // 版本计数器：避免 Resource 缓存同一 key，确保每次点击都重新获取
    let detail_ver: RwSignal<u32> = RwSignal::new(0);
    let detail_open = Signal::derive(move || selected_id.get().is_some());
    let detail_data = Resource::new(
        move || (selected_id.get(), detail_ver.get()),
        |(id, _)| async move {
            match id.filter(|s| !s.is_empty()) {
                Some(id) => get_football_and_increment(id).await,
                _ => Ok(None),
            }
        },
    );
    let detail_close = detail_close_nav!(selected_id, i18n, "");
    let on_card_click = detail_open_nav!(selected_id, detail_ver, i18n, "/footballs/");

    view! {
        <Title text=move || site_title!(i18n)/>
        <main class={WIDE}>
            <div class="mb-10 text-center">
                <p class={[TEXT_SUBTLE, "text-sm", "max-w-2xl", "mx-auto"].join(" ")}>
                    <a style="white-space: pre-line" href="https://github.com/bipou/ftbsite" target="_blank" rel="noopener noreferrer">
                        {move || t!(i18n, site_intro)}
                    </a>
                </p>
                <p class={[TEXT_WARN, "mt-3", "max-w-2xl", "mx-auto"].join(" ")}>
                    {move || t!(i18n, site_warn)}
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex justify-center py-16">
                    <div class="text-gray-400 text-sm">{move || t!(i18n, loading)}</div>
                </div>
            }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either::Left(view! { <p class="text-red-500 text-center py-8">{e.to_string()}</p> }),
                    Ok(d) => {
                        let cb = on_card_click.clone();
                        Either::Right(view! {
                            <HomeSection ana_type=0 footballs=d.user on_click=cb/>
                            <HomeSection ana_type=1 footballs=d.pre on_click=cb/>
                            <HomeSection ana_type=2 footballs=d.post on_click=cb/>
                        })
                    },
                })}
            </Suspense>

            // ── 详情底部滑出面板 ──────────────────────────────────────────
            <SlidePanel open=detail_open on_close=detail_close panel_class=Signal::derive(|| SLIDE_SIZED_LG.to_string())>
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-16">
                        <div class="text-gray-400">{move || t!(i18n, loading)}</div>
                    </div>
                }>
                    {move || detail_data.get().map(|result| match result {
                        Err(e) => Either3::Left(view! {
                            <p class="text-red-500 text-center py-8">{e.to_string()}</p>
                        }),
                        Ok(None) => Either3::Right(Either::Left(view! {
                            <div class=EMPTY>
                                <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                            </div>
                        })),
                        Ok(Some(f)) => Either3::Right(Either::Right(view! {
                            <FootballDetail f=f/>
                        })),
                    })}
                </Suspense>
            </SlidePanel>
        </main>
    }
}
