use crate::i18n::{t, use_i18n};
use crate::page_title;
use crate::shared::constant::{BADGE_BLUE_NO_UL, EMPTY, GRID_3, H1, MAIN, NO_DATA, WIDE};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::{Footer, Nav, Pagination, UserIntro, UserTopics};
use crate::models::{User, UserSummary, UsersResult};
use crate::shared::common::Either3;
use crate::shared::locale::{LocaleA, use_locale};

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
pub async fn get_user_profile(username: String) -> Result<Option<User>, ServerFnError> {
    use crate::server::user_db;
    user_db::get_user_by_username(&username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let query = use_query_map();
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };

    let data = Resource::new_blocking(move || from(), |f| async move { get_users_page(f).await });

    view! {
        <Title text=move || page_title!(i18n, users_list)/>
        <Nav/>
        <main class={WIDE}>
            <h1 class={H1}>
                {move || t!(i18n, users_list)}
            </h1>
            <Suspense fallback=move || view! { <div class={format!("{} text-gray-400", EMPTY)}>{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either::Left(view! {
                        <p class="text-red-500 text-center">{e.to_string()}</p>
                    }),
                    Ok(d) => {
                        let pi = d.page_info.clone();
                        Either::Right(view! {
                            <div class={format!("{} mb-8", GRID_3)}>
                                {d.items.into_iter().map(|user| {
                                    let UserSummary { username, updated_at, keywords, .. } = user;
                                    let url = format!("/{}/users/{}", loc_str.get(), username);
                                    let initial = username.chars().next().unwrap_or('?');
                                    view! {
                                        <div class=CARD_BLOCK_NO_UL>
                                            <div class="flex items-start gap-3">
                                                <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-2xl shrink-0 mt-1">
                                                    {initial.to_string()}
                                                </div>
                                                <div class="min-w-0 flex-1">
                                                    <a href=url target="_blank" rel="noopener noreferrer" class="text-2xl font-bold text-gray-800 dark:text-gray-100 truncate no-underline hover:underline hover:text-blue-600">{username}</a>
                                                    <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)}{updated_at}</p>
                                                    {if !keywords.is_empty() {
                                                        Either::Left(view! {
                                                            <div class="flex flex-wrap gap-1 mt-1">
                                                                {keywords.iter().take(8).map(|topic| {
                                                                    let kid = crate::shared::common::record_key(&topic.id).to_string();
                                                                    let url = format!("/{}/footballs?topic={}", loc_str.get(), kid);
                                                                    let name = topic.name.clone();
                                                                    view! {
                                                                        <a href=url class=format!("text-sm {}", BADGE_BLUE_NO_UL)>{name}</a>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>
                                                        })
                                                    } else {
                                                        Either::Right(())
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <Pagination page_info=pi base_url=format!("/{}/users", loc_str.get())/>
                        })
                    }
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}

#[component]
pub fn UserProfilePage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let username = move || params.read().get("username").unwrap_or_default();

    let data = Resource::new_blocking(
        move || username(),
        |u| async move { get_user_profile(u).await },
    );

    view! {
        <Nav/>
        <main class={MAIN}>
            <Suspense fallback=move || view! { <div class={format!("{} text-gray-400", EMPTY)}>{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either3::Left(view! {
                        <p class="text-red-500 text-center">{e.to_string()}</p>
                    }),
                    Ok(None) => Either3::Right(Either::Left(view! {
                        <div class={EMPTY}>
                            <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                            <LocaleA href="/users" class="btn-primary">{move || t!(i18n, go_list)}</LocaleA>
                        </div>
                    })),
                    Ok(Some(user)) => {
                        let User { username, created_at, updated_at, introduction_html, keywords, topics, .. } = user;
                        let intro_html = introduction_html;
                        let initial = username.chars().next().unwrap_or('?');
                        let title = format!("{} – {}", username, site_title!(i18n));
                        Either3::Right(Either::Right(view! {
                            <Title text=title/>

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
        </main>
        <Footer/>
    }
}
