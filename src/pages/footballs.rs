use crate::detail_close_nav;
use crate::i18n::{t, use_i18n};
use crate::page_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::components::{ArticleCard, CategorySelect, FootballCard, Pagination, SlidePanel};
use crate::models::FootballsResult;
use crate::pages::football::{FootballDetail, get_football_and_increment};
use crate::pages::write::get_all_categories;

use crate::shared::common::{Either3, record_key};
use crate::shared::constant::{EMPTY, GRID_3, NO_DATA, SLIDE_SIZED_LG, TEXT_WARN, WIDE};

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
    let params = use_params_map();
    let navigate = use_navigate();

    let initial_id = params.read_untracked().get("id").filter(|s| !s.is_empty());
    let selected_id: RwSignal<Option<String>> = RwSignal::new(initial_id);

    // 从 URL :id 设置 selected_id（跟踪后续变化）
    Effect::new(move |_| {
        selected_id.set(params.read().get("id").filter(|s| !s.is_empty()));
    });

    // 详情面板信号
    let detail_open = Signal::derive(move || selected_id.get().is_some());
    let detail_data = Resource::new_blocking(
        move || selected_id.get(),
        |id| async move {
            match id.filter(|s| !s.is_empty()) {
                Some(id) => get_football_and_increment(id).await,
                _ => Ok(None),
            }
        },
    );
    let detail_close = detail_close_nav!(selected_id, i18n, "/footballs");
    let on_card_click = {
        let navigate = navigate.clone();
        Callback::new(move |fid: String| {
            navigate(
                &["/", &i18n.get_locale().to_string(), "/footballs/", &fid].join(""),
                Default::default(),
            );
            selected_id.set(Some(fid));
        })
    };

    // 路由参数 → 稳定信号（params Memo 路由切换时释放，不能直接捕获）
    let from_sig = RwSignal::new(1i64);
    let filter_sig: RwSignal<(String, String)> = RwSignal::new((String::new(), String::new()));
    Effect::new(move |_| {
        let p = params.read();
        from_sig.set(p.get("from").and_then(|v| v.parse().ok()).unwrap_or(1));
        if let Some(cid) = p.get("cid").filter(|s| !s.is_empty()) {
            filter_sig.set(("category".into(), cid));
        } else if let Some(tid) = p.get("tid").filter(|s| !s.is_empty()) {
            filter_sig.set(("topic".into(), tid));
        } else {
            filter_sig.set((String::new(), String::new()));
        }
    });

    let cats_res = Resource::new(|| (), |_| get_all_categories());

    let footballs_res = Resource::new_blocking(
        move || (from_sig.get(), filter_sig.get().0, filter_sig.get().1),
        |(f, fi, fid)| async move { get_footballs_page(f, fi, fid).await },
    );

    // h1 标题后缀和页面标题
    let heading_suffix = RwSignal::new(String::new());
    Effect::new(move |_| {
        heading_suffix.set(match filter_sig.get().0.as_str() {
            "category" => {
                let fid = filter_sig.get().1;
                if fid.is_empty() {
                    String::new()
                } else {
                    cats_res
                        .get()
                        .and_then(|r| r.ok())
                        .and_then(|cats| {
                            cats.iter().find(|c| record_key(&c.id) == fid).map(|c| {
                                let name = c
                                    .name
                                    .get(&i18n.get_locale().to_string())
                                    .cloned()
                                    .unwrap_or_default();
                                [" | ", &name].join("")
                            })
                        })
                        .unwrap_or_default()
                }
            }
            "topic" => String::new(),
            _ => String::new(),
        });
    });

    view! {
        <Title text=move || page_title!(i18n, football_list)/>
                <main class=WIDE>
            <p class={[TEXT_WARN, "text-center", "mb-2"].join(" ")}>
                {move || t!(i18n, site_warn)}
            </p>
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">
                    {move || t!(i18n, football_list)}
                    <Suspense fallback=|| "">
                                            {move || heading_suffix.get()}
                    </Suspense>
                </h1>
                <a
                    href=move || ["/", &i18n.get_locale().to_string(), "/footballs/analysis/new"].join("")
                    class="inline-flex items-center justify-center bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg px-6 py-3 text-lg transition-colors no-underline"
                >
                    {move || t!(i18n, write_article)}
                </a>
            </div>
            // ── Horizontal category filter bar ───────────────────────────
            <div class="mb-6 pr-8">
                <div class="flex">
                    <span class="form-label shrink-0 mr-1">
                        {move || t!(i18n, football_category)}
                    </span>
                    <div class="flex flex-wrap gap-2">
                        <a href=move || ["/", &i18n.get_locale().to_string(), "/footballs"].join("")
                                                   class="text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700">
                                                    {move || t!(i18n, all)}
                                                </a>
                                                <a href=move || ["/", &i18n.get_locale().to_string(), "/footballs?picks"].join("")
                                                    class="text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50">
                                                    {move || t!(i18n, status_picks)}
                                                </a>
                                                <a href=move || ["/", &i18n.get_locale().to_string(), "/footballs?hot"].join("")
                            class="text-red-500 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50">
                            {move || t!(i18n, status_hot)}
                        </a>
                        <Suspense fallback=|| ()>
                                                    {move || cats_res.get().map(|r| r.ok()).flatten().map(|cats| {
                                view! { <CategorySelect all=cats expandable=true/> }
                            })}
                        </Suspense>
                    </div>
                </div>
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
                            let pi = data.page_info;
                            let base = match filter_sig.get().0.as_str() {
                                "category" => ["/", &i18n.get_locale().to_string(), "/footballs/category/", &filter_sig.get().1].join(""),
                                "topic" => ["/", &i18n.get_locale().to_string(), "/footballs/topic/", &filter_sig.get().1].join(""),
                                _ => ["/", &i18n.get_locale().to_string(), "/footballs"].join(""),
                            };
                            if data.items.is_empty() {
                                Either3::Right(Either::Left(view! {
                                    <div class=EMPTY>
                                        <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                                    </div>
                                }))
                            } else {
                                let cb = on_card_click.clone();
                                Either3::Right(Either::Right(view! {
                                    <div class={GRID_3}>
                                        {data.items.into_iter().map(|f| {
                                            let at = f.ana_type;
                                            let cb = cb.clone();
                                            view! {
                                                {match at == 0 {
                                                    true => Either::Left(view! { <ArticleCard football=f on_click=cb/> }),
                                                    false => Either::Right(view! { <FootballCard football=f on_click=cb/> }),
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
