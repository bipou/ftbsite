use crate::i18n::{t, t_display, use_i18n};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use serde::{Deserialize, Serialize};

use crate::components::{ArticleCard, FootballCard, Footer, Nav};
use crate::models::Football;

use crate::shared::constant::{EMPTY, GRID_3, NO_DATA, TEXT_SUBTLE, TEXT_WARN, WIDE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeData {
    pub user: Vec<Football>,
    pub pre: Vec<Football>,
    pub post: Vec<Football>,
}

#[server]
pub async fn get_home_data() -> Result<HomeData, ServerFnError> {
    use crate::server::football_db;
    let all = football_db::get_home_footballs(12)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let mut user = Vec::new();
    let mut pre = Vec::new();
    let mut post = Vec::new();
    for f in all {
        match f.ana_type {
            0 if user.len() < 3 => user.push(f),
            1 if pre.len() < 3 => pre.push(f),
            2 if post.len() < 3 => post.push(f),
            _ => {}
        }
    }
    Ok(HomeData { user, pre, post })
}

#[component]
fn HomeSection(ana_type: u8, footballs: Vec<Football>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section class="mb-12">
            <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-200 border-b border-blue-200 dark:border-blue-800 pb-2 mb-4 flex items-center gap-2">
                <span class="text-blue-500">{match ana_type { 0 => "✍️", 1 => "📊", _ => "📋" }}</span>
                {match ana_type {
                    0 => {t_display!(i18n, user_analysis).to_string()},
                    1 => {t_display!(i18n, pre_match_analysis).to_string()},
                    _ => {t_display!(i18n, post_match_review).to_string()},
                }}
            </h2>
            {if footballs.is_empty() {
                Either::Left(view! {
                    <div class=EMPTY>
                        <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                    </div>
                })
            } else {
                Either::Right(view! {
                    <div class={GRID_3}>
                        {footballs.into_iter().map(|f| {
                            if f.ana_type == 0 {
                                Either::Left(view! { <ArticleCard football=f/> })
                            } else {
                                Either::Right(view! { <FootballCard football=f/> })
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                })
            }}
        </section>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();
    let data = Resource::new_blocking(|| (), |_| get_home_data());

    view! {
        <Title text=move || site_title!(i18n)/>
        <Nav/>
        <main class={WIDE}>
            <div class="mb-10 text-center">
                <p class={format!("{} text-sm max-w-2xl mx-auto", TEXT_SUBTLE)}>
                    <a style="white-space: pre-line" href="https://github.com/bipou/football-site" target="_blank" rel="noopener noreferrer">
                        {move || t!(i18n, site_intro)}
                    </a>
                </p>
                <p class={format!("{} mt-3 max-w-2xl mx-auto", TEXT_WARN)}>
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
                    Ok(d) => Either::Right(view! {
                        <HomeSection ana_type=0 footballs=d.user/>
                        <HomeSection ana_type=1 footballs=d.pre/>
                        <HomeSection ana_type=2 footballs=d.post/>
                    }),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
