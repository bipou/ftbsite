use crate::i18n::{t, t_display, use_i18n};
use crate::shared::locale::use_locale;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

use crate::components::{ArticleCard, FootballCard, Footer, Nav, Pagination};
use crate::models::{Category, FootballsResult};

use crate::shared::common::{Either3, record_key};
use crate::shared::constant::{EMPTY, GRID_3, NO_DATA, TEXT_WARN, WIDE};

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
    let mut cats = category_db::get_categories_by_levels(&[1, 2])
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    // pinned 优先，其余按 level ASC
    cats.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(a.level.cmp(&b.level)));
    Ok(cats)
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
    let loc_str = use_locale();
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

    let footballs_res = Resource::new_blocking(
        move || (from(), filter(), filter_id()),
        |(f, fi, fid)| async move { get_footballs_page(f, fi, fid).await },
    );

    // h1 和页面标题的筛选后缀，统一定义
    let heading_suffix = move || match filter().as_str() {
        "picks" => format!(" | {}", t_display!(i18n, status_picks)),
        "hot" => format!(" | {}", t_display!(i18n, status_hot)),
        "topic" | "category" => {
            let fid = filter_id();
            if fid.is_empty() {
                String::new()
            } else {
                cats_res
                    .get()
                    .and_then(|r| r.ok())
                    .and_then(|cats| {
                        cats.iter().find(|c| record_key(&c.id) == fid).map(|c| {
                            let name = c.name.get(&loc_str.get()).cloned().unwrap_or_default();
                            format!(" | {}", name)
                        })
                    })
                    .unwrap_or_default()
            }
        }
        _ => String::new(),
    };

    // 页面标题：footballs_list | 筛选名 – site_name | site_slogan
    let title_text = move || {
        format!(
            "{}{} – {} | {}",
            t_display!(i18n, footballs_list),
            heading_suffix(),
            t_display!(i18n, site_name),
            t_display!(i18n, site_slogan),
        )
    };

    view! {
        <Title text=title_text/>
        <Nav/>
        <main class=WIDE>
            <p class={format!("{} text-center mb-2", TEXT_WARN)}>
                {move || t!(i18n, site_warn)}
            </p>
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">
                    {move || t!(i18n, footballs_list)}
                    {heading_suffix}
                </h1>
                <a
                    href=move || format!("/{}/footballs/share-analysis", loc_str.get())
                    class="inline-flex items-center justify-center bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg px-6 py-3 text-lg transition-colors no-underline"
                >
                    {move || t!(i18n, write_article)}
                </a>
            </div>
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
                                    <div class=EMPTY>
                                        <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                                    </div>
                                }))
                            } else {
                                Either3::Right(Either::Right(view! {
                                    <div class={GRID_3}>
                                        {data.items.into_iter().map(|f| {
                                            let at = f.ana_type;
                                            view! {
                                                {if at == 0 {
                                                    Either::Left(view! { <ArticleCard football=f/> })
                                                } else {
                                                    Either::Right(view! { <FootballCard football=f/> })
                                                }}
                                            }
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
