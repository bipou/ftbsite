use crate::i18n::{t, use_i18n};
use crate::shared::locale::use_locale_str;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

use crate::components::{FootballCard, Footer, Nav, Pagination};
use crate::models::{Category, FootballsResult};

use crate::page_title;
use crate::shared::common::Either3;
use crate::shared::constant::{EMPTY, GRID_3, WIDE};

// ── Server functions ──────────────────────────────────────────────────────────

/// Returns a random published football ID for the "random" nav button.
#[server]
pub async fn get_random_id() -> Result<Option<String>, ServerFnError> {
    use crate::server::football_db;
    football_db::get_random_football_id()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_sidebar_categories() -> Result<Vec<Category>, ServerFnError> {
    use crate::server::category_db;
    category_db::get_categories_by_levels(&[1, 2])
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FootballsFilter {
    All,
    Picks,
    Hot,
    ByCategory(String),
    ByTopic(String),
}

#[server]
pub async fn get_footballs_page(
    from: i64,
    filter: String,
    filter_id: String,
) -> Result<FootballsResult, ServerFnError> {
    use crate::server::football_db;
    use crate::shared::common::into_rid;
    let res = match filter.as_str() {
        "picks" => football_db::get_footballs(from, 3, 4).await,
        "hot" => football_db::get_footballs(from, 2, 4).await,
        "category" => {
            let rid = into_rid(&filter_id, "categories");
            football_db::get_footballs_by_category(&rid, from).await
        }
        "topic" => {
            let rid = into_rid(&filter_id, "topics");
            football_db::get_footballs_by_topic(&rid, from).await
        }
        _ => football_db::get_footballs(from, 1, 4).await,
    };
    res.map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Page component ────────────────────────────────────────────────────────────

#[component]
pub fn FootballsPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale_str();
    let query = use_query_map();

    // Reactive query params
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };
    // /footballs?topic=xxx  /footballs?category=xxx  /footballs?picks  /footballs?hot
    let filter = move || {
        let q = query.read();
        if q.get("topic").is_some() {
            "topic".to_string()
        } else if q.get("category").is_some() {
            "category".to_string()
        } else if q.get("picks").is_some() {
            "picks".to_string()
        } else if q.get("hot").is_some() {
            "hot".to_string()
        } else {
            String::new()
        }
    };
    let filter_id = move || {
        let q = query.read();
        q.get("topic")
            .or_else(|| q.get("category"))
            .unwrap_or_default()
    };

    let cats_res = Resource::new(|| (), |_| get_sidebar_categories());

    let footballs_res = Resource::new(
        move || (from(), filter(), filter_id()),
        |(f, fi, fid)| async move { get_footballs_page(f, fi, fid).await },
    );

    let filter_label = move || match filter().as_str() {
        "picks" => Either3::Left(t!(i18n, status_picks)),
        "hot" => Either3::Right(Either::Left(t!(i18n, status_hot))),
        _ => Either3::Right(Either::Right(t!(i18n, footballs_list))),
    };

    view! {
        <Title text=move || page_title!(i18n, footballs_list)/>
        <Nav/>
        <main class=WIDE>
            <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-4">
                {filter_label}
            </h1>

            // ── Horizontal category filter bar ───────────────────────────
            <div class="mb-6">
                <nav class="cat-bar flex flex-wrap items-center gap-x-2 gap-y-1">
                    <span class="text-sm text-gray-400 dark:text-gray-500 shrink-0 mr-1">
                        {move || t!(i18n, footballs_filter_category)}
                    </span>
                    <a href=move || format!("/{}/footballs", loc_str.get())
                       class="text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700">
                        {move || t!(i18n, all)}
                    </a>
                    <a href=move || format!("/{}/footballs?picks", loc_str.get())
                        class="text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50">
                        {move || t!(i18n, status_picks)}
                    </a>
                    <a href=move || format!("/{}/footballs?hot", loc_str.get())
                        class="text-red-500 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50">
                        {move || t!(i18n, status_hot)}
                    </a>
                    <Suspense fallback=|| ()>
                        {move || cats_res.get().map(|r| r.ok()).flatten().map(|cats| {
                            view! {
                                {cats.into_iter().map(|cat| {
                                    let kid = crate::shared::common::record_key(&cat.id).to_string();
                                    let url = format!("/{}/footballs?category={}", loc_str.get(), kid);
                                    let cat_name = cat.name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default();
                                    view! {
                                        <a href=url class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700">
                                            {cat_name}
                                        </a>
                                    }
                                }).collect::<Vec<_>>()}
                            }
                        })}
                    </Suspense>
                </nav>
            </div>

            // ── Main content ─────────────────────────────────────────────
            <div>
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-16">
                        <div class="text-gray-400">{move || t!(i18n, loading)}</div>
                    </div>
                }>
                    {move || footballs_res.get().map(|result| match result {
                        Err(e) => Either3::Left(view! {
                            <p class="text-red-500 py-8 text-center">{e.to_string()}</p>
                        }),
                        Ok(data) => {
                            let pi = data.page_info.clone();
                            let base = match filter().as_str() {
                                "topic" | "category" => format!("/{}/footballs?{}={}", loc_str.get(), filter(), filter_id()),
                                "picks" | "hot" => format!("/{}/footballs?{}", loc_str.get(), filter()),
                                _ => format!("/{}/footballs", loc_str.get()),
                            };
                            if data.items.is_empty() {
                                Either3::Right(Either::Left(view! {
                                    <p class=format!("text-gray-400 {}", EMPTY)>
                                        {move || t!(i18n, no_match)}
                                    </p>
                                }))
                            } else {
                                Either3::Right(Either::Right(view! {
                                    <div class={GRID_3}>
                                        {data.items.into_iter().map(|f| view! {
                                            <FootballCard football=f/>
                                        }).collect::<Vec<_>>()}
                                    </div>
                                    <Pagination page_info=pi base_url=base/>
                                }))
                            }
                        }
                    })}
                </Suspense>
            </div>
        </main>
        <Footer/>
    }
}
