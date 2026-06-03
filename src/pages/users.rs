use crate::detail_close_nav;
use crate::i18n::{t, use_i18n};
use crate::page_title;
use crate::shared::common::{Either3, record_key};
use crate::shared::constant::{BADGE_BLUE_NO_UL, EMPTY, GRID_3, H1, NO_DATA, SLIDE_SIZED_LG, WIDE};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::{Pagination, SlidePanel, UserIntro, UserTopics};
use crate::models::{User, UserSummary, UsersResult};
use crate::shared::locale::use_locale;

const CARD_BLOCK_NO_UL: &str = "card p-4 block no-underline hover:shadow-md transition-shadow";

// ── Server functions ───────────────────────────────────────────────────────────

#[server]
pub async fn get_users_page(from: i64) -> Result<UsersResult, ServerFnError> {
    use crate::server::user_db;
    user_db::get_users(from)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_user_by_id(user_id: String) -> Result<Option<User>, ServerFnError> {
    use crate::server::user_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&user_id, "users");
    user_db::get_user_by_id(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let params = use_params_map();
    let query = use_query_map();
    let navigate = leptos_router::hooks::use_navigate();
    let from_sig = RwSignal::new(1i64);
    Effect::new(move |_| {
        from_sig.set(
            query
                .read()
                .get("from")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        );
    });

    let data = Resource::new_blocking(
        move || from_sig.get(),
        |f| async move { get_users_page(f).await },
    );

    // 从 URL :id 初始化并同步 selected_id
    let selected_id: RwSignal<Option<String>> = {
        let initial = params.read_untracked().get("id").filter(|s| !s.is_empty());
        RwSignal::new(initial)
    };
    Effect::new(move |_| {
        selected_id.set(params.read().get("id").filter(|s| !s.is_empty()));
    });

    let detail_open = Signal::derive(move || selected_id.get().is_some());
    let detail_data = Resource::new(
        move || selected_id.get(),
        |id| async move {
            match id.filter(|s| !s.is_empty()) {
                Some(id) => get_user_by_id(id).await,
                None => Ok(None),
            }
        },
    );
    let detail_close = detail_close_nav!(selected_id, i18n, "/users");
    let on_card_click = {
        let navigate = navigate.clone();
        Callback::new(move |uid: String| {
            let kid = record_key(&uid).to_string();
            navigate(
                &["/", &i18n.get_locale().to_string(), "/users/", &kid].join(""),
                Default::default(),
            );
            selected_id.set(Some(kid));
        })
    };

    view! {
        <Title text=move || page_title!(i18n, users_list)/>
        <main class={WIDE}>
            <h1 class={H1}>
                {move || t!(i18n, users_list)}
            </h1>
            <Suspense fallback=move || view! { <div class={[EMPTY, "text-gray-400"].join(" ")}>{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either::Left(view! {
                        <p class="text-red-500 text-center">{e.to_string()}</p>
                    }),
                    Ok(d) => {
                        let pi = d.page_info;
                        Either::Right(view! {
                            <div class={[GRID_3, "mb-8"].join(" ")}>
                                {d.items.into_iter().map(|user| {
                                    let UserSummary { id, username, updated_at, keywords, .. } = user;
                                    let initial = username.chars().next().unwrap_or('?');
                                    let kid = record_key(&id).to_string();
                                    let cb = on_card_click.clone();
                                    view! {
                                        <button class=[CARD_BLOCK_NO_UL, "w-full text-left border-0 cursor-pointer"].join(" ") on:click={
                                            let cb = cb.clone();
                                            let kid = kid.clone();
                                            move |_| cb.run(kid.clone())
                                        }>
                                            <div class="flex items-start gap-3">
                                                <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-2xl shrink-0 mt-1">
                                                    {initial.to_string()}
                                                </div>
                                                <div class="min-w-0 flex-1">
                                                    <span class="text-2xl font-bold text-gray-800 dark:text-gray-100 truncate">{username}</span>
                                                    <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)}{updated_at}</p>
                                                    {match keywords.is_empty() {
                                                        false => Either::Left(view! {
                                                            <div class="flex flex-wrap gap-1 mt-1">
                                                                {keywords.iter().take(8).map(|topic| {
                                                                                                                                    let kid = crate::shared::common::record_key(&topic.id).to_string();
                                                                                                                                    let name = topic.name.clone();
                                                                                                                                    let url = ["/", &loc_str.get(), "/footballs/topic/", &kid].join("");
                                                                    view! {
                                                                        <a href=url class=["text-sm", BADGE_BLUE_NO_UL].join(" ")>{name}</a>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>
                                                        }),
                                                        true => Either::Right(()),
                                                    }}
                                                </div>
                                            </div>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <Pagination page_info=pi base_url=["/", &loc_str.get(), "/users"].join("")/>
                        })
                    }
                })}
            </Suspense>

            // ── 用户详情底部滑出面板 ──────────────────────────────────
            <SlidePanel open=detail_open on_close=detail_close panel_class=Signal::derive(|| SLIDE_SIZED_LG.to_string())>
                <Suspense fallback=move || view! {
                    <div class=[EMPTY, "text-gray-400"].join(" ")>{move || t!(i18n, loading)}</div>
                }>
                    {move || detail_data.get().map(|result| match result {
                        Err(e) => Either3::Left(view! { <p class="text-red-500 text-center">{e.to_string()}</p> }),
                        Ok(None) => Either3::Right(Either::Left(view! {
                            <div class=EMPTY>
                                <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                            </div>
                        })),
                        Ok(Some(user)) => {
                            let User { username, created_at, updated_at, introduction_html, keywords, topics, .. } = user;
                            let intro_html = introduction_html;
                            let initial = username.chars().next().unwrap_or('?');
                            Either3::Right(Either::Right(view! {
                                <div class="card p-6 mb-6">
                                    <div class="flex items-start gap-3">
                                        <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-2xl shrink-0 mt-1">
                                            {initial.to_string()}
                                        </div>
                                        <div class="min-w-0 flex-1">
                                            <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">{username}</h1>
                                            <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, registration_time)}{created_at}</p>
                                            <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)}{updated_at}</p>
                                        </div>
                                    </div>
                                </div>
                                <UserIntro intro_html=intro_html/>
                                <UserTopics keywords=keywords topics=topics/>
                            }))
                        }
                    })}
                </Suspense>
            </SlidePanel>
        </main>
    }
}
