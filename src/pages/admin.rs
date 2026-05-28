use crate::i18n::{t, t_display, use_i18n};
use crate::page_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::{Footer, Nav, Pagination, UserIntro, UserTopics};
use crate::models::{Football, FootballsResult, User, UsersResult};

use crate::shared::common::{Either3, Either7};
use crate::shared::constant::{
    EMPTY, GRID_2, GRID_3, H1, HOVER_SHADOW, MAIN, NO_DATA, NO_UNDERLINE,
};
use crate::shared::locale::{LocaleA, use_locale};

// ── 服务端函数 ──────────────────────────────────────────────────────────

/// 查询当前用户 status，用于 AdminGuard
#[server]
pub async fn get_my_status() -> Result<Option<i8>, ServerFnError> {
    use crate::server::auth::{decode_jwt, get_cookie_value};
    use crate::server::user_db;
    use axum::http::HeaderMap;

    let headers: HeaderMap = leptos_axum::extract().await?;
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let token = match get_cookie_value(cookie, "fs_token") {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(None),
    };

    let claims = decode_jwt(&token).map_err(|e| ServerFnError::new(e))?;
    user_db::get_user_status_by_username(&claims.username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn admin_get_user(username: String) -> Result<Option<User>, ServerFnError> {
    use crate::server::user_db;
    user_db::get_user_by_username(&username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_admin_users(from: i64) -> Result<UsersResult, ServerFnError> {
    use crate::server::user_db;
    user_db::get_admin_users(from)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn admin_update_user_status(user_id: String, status: i8) -> Result<(), ServerFnError> {
    use crate::server::user_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&user_id, "users");
    user_db::update_user_status(&rid, status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_admin_footballs(from: i64) -> Result<FootballsResult, ServerFnError> {
    use crate::server::football_db;
    football_db::get_footballs_admin(from)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn admin_update_status(football_id: String, status: i8) -> Result<(), ServerFnError> {
    use crate::server::football_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&football_id, "footballs");
    football_db::update_football_status(&rid, status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn admin_get_football(id: String) -> Result<Option<Football>, ServerFnError> {
    use crate::server::football_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&id, "footballs");
    football_db::get_football_by_id(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── 辅助 ────────────────────────────────────────────────────────────────

fn user_status_cls(status: i8) -> &'static str {
    match status {
        1..=10 => "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300",
        0 => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300",
        _ => "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300",
    }
}

// 管理按钮 CSS 常量
const UBTN_GREEN: &str = "bg-green-100 hover:bg-green-200 text-green-700 dark:bg-green-900/30 dark:hover:bg-green-900/50 dark:text-green-300";
const UBTN_YELLOW: &str = "bg-yellow-100 hover:bg-yellow-200 text-yellow-700 dark:bg-yellow-900/30 dark:hover:bg-yellow-900/50 dark:text-yellow-300";
const UBTN_RED: &str = "bg-red-100 hover:bg-red-200 text-red-700 dark:bg-red-900/30 dark:hover:bg-red-900/50 dark:text-red-300";
const UBTN_DRAFT: &str = "bg-gray-200 text-gray-500 dark:bg-gray-700 dark:text-gray-300";
const UBTN_DELETED: &str =
    "bg-gray-50 text-gray-300 line-through dark:bg-gray-900 dark:text-gray-600";
const FBTN_ORANGE: &str = "bg-orange-100 hover:bg-orange-200 text-orange-700 dark:bg-orange-900/30 dark:hover:bg-orange-900/50 dark:text-orange-300";
const FBTN_INDIGO: &str = "bg-indigo-100 hover:bg-indigo-200 text-indigo-700 dark:bg-indigo-900/30 dark:hover:bg-indigo-900/50 dark:text-indigo-300";
const FBTN_BLUE: &str = "bg-blue-100 hover:bg-blue-200 text-blue-700 dark:bg-blue-900/30 dark:hover:bg-blue-900/50 dark:text-blue-300";
const FBTN_SUBMIT: &str = "bg-gray-100 hover:bg-gray-200 text-gray-600 dark:bg-gray-800 dark:hover:bg-gray-700 dark:text-gray-200";

/// 用户状态标签（激活/未激活/禁止）— 管理列表和详情页共用
#[component]
fn UserStatusBadge(status: i8) -> impl IntoView {
    let i18n = use_i18n();
    let st_cls = user_status_cls(status);
    view! {
        <span class=format!("text-xs px-1.5 py-0.5 rounded {}", st_cls)>
            {move || match status {
                1..=10 => Either3::Left(t!(i18n, status_active)),
                0 => Either3::Right(Either::Left(t!(i18n, status_inactive))),
                _ => Either3::Right(Either::Right(t!(i18n, status_banned))),
            }}
        </span>
    }
}

/// 用户 3 状态操作按钮 — 列表和详情页共用
#[component]
fn UserStatusButtons(
    id: String,
    initial_status: i8,
    action: ServerAction<AdminUpdateUserStatus>,
) -> impl IntoView {
    let i18n = use_i18n();
    let status = RwSignal::new(initial_status);
    let feedback = RwSignal::new(None::<String>);

    let onclick = move |s: i8| {
        let uid = id.clone();
        let act = action;
        move |_| {
            status.set(s);
            let label = match s {
                1 => t_display!(i18n, status_active).to_string(),
                0 => t_display!(i18n, status_inactive).to_string(),
                _ => t_display!(i18n, status_banned).to_string(),
            };
            feedback.set(Some(format!("{}: {}", t_display!(i18n, admin_status_updated).to_string(), label)));
            #[cfg(feature = "hydrate")]
            set_timeout(move || feedback.set(None), std::time::Duration::from_secs(3));
            act.dispatch(AdminUpdateUserStatus {
                user_id: uid.clone(),
                status: s,
            });
        }
    };

    view! {
        <div>
            <div class="flex gap-1 flex-wrap">
            {[
                (1i8, "status_active", UBTN_GREEN),
                (0, "status_inactive", UBTN_YELLOW),
                (-1, "status_banned", UBTN_RED),
            ].into_iter().map(|(s, _key, cls)| {
                let hl = move || if status.get() == s { " ring-2 ring-blue-400 ring-offset-1" } else { "" };
                view! {
                    <button
                        type="button"
                        class=move || format!("text-xs px-2 py-1 rounded transition-colors {}{}", cls, hl())
                        on:click=onclick(s)
                    >
                        {move || match s {
                            1 => Either3::Left(t!(i18n, status_active)),
                            0 => Either3::Right(Either::Left(t!(i18n, status_inactive))),
                            _ => Either3::Right(Either::Right(t!(i18n, status_banned))),
                        }}
                    </button>
                }
            }).collect::<Vec<_>>()}
            </div>
            {move || feedback.get().map(|msg| view! {
                <p class="text-green-500 text-xs mt-1">{msg}</p>
            })}
        </div>
    }
}

/// 足球 7 状态操作按钮 — 列表和详情页共用
#[component]
fn FootballStatusButtons(
    id: String,
    initial_status: i8,
    action: ServerAction<AdminUpdateStatus>,
) -> impl IntoView {
    let i18n = use_i18n();
    let status = RwSignal::new(initial_status);
    let feedback = RwSignal::new(None::<String>);

    let onclick = move |s: i8| {
        let fid = id.clone();
        let act = action;
        move |_| {
            status.set(s);
            let label = match s {
                4 => t_display!(i18n, status_both).to_string(),
                3 => t_display!(i18n, status_picks).to_string(),
                2 => t_display!(i18n, status_hot).to_string(),
                1 => t_display!(i18n, status_publish).to_string(),
                0 => t_display!(i18n, status_submit).to_string(),
                -1 => t_display!(i18n, status_draft).to_string(),
                _ => t_display!(i18n, status_deleted).to_string(),
            };
            feedback.set(Some(format!("{}: {}", t_display!(i18n, admin_status_updated).to_string(), label)));
            #[cfg(feature = "hydrate")]
            set_timeout(move || feedback.set(None), std::time::Duration::from_secs(3));
            act.dispatch(AdminUpdateStatus {
                football_id: fid.clone(),
                status: s,
            });
        }
    };

    view! {
        <div>
            <div class="flex gap-1 flex-wrap">
            {[
                (4i8, "status_both", UBTN_RED),
                (3, "status_picks", FBTN_ORANGE),
                (2, "status_hot", FBTN_INDIGO),
                (1, "status_publish", FBTN_BLUE),
                (0, "status_submit", FBTN_SUBMIT),
                (-1, "status_draft", UBTN_DRAFT),
                (-2, "status_deleted", UBTN_DELETED),
            ].into_iter().map(|(s, _key, cls)| {
                let hl = move || if status.get() == s { " ring-2 ring-blue-400 ring-offset-1" } else { "" };
                view! {
                    <button
                        type="button"
                        class=move || format!("text-xs px-2 py-1 rounded transition-colors {}{}", cls, hl())
                        on:click=onclick(s)
                    >
                        {move || match s {
                            4 => Either7::Left(t!(i18n, status_both)),
                            3 => Either7::Right(Either::Left(t!(i18n, status_picks))),
                            2 => Either7::Right(Either::Right(Either::Left(t!(i18n, status_hot)))),
                            1 => Either7::Right(Either::Right(Either::Right(Either::Left(t!(i18n, status_publish))))),
                            0 => Either7::Right(Either::Right(Either::Right(Either::Right(Either::Left(t!(i18n, status_submit)))))),
                            -1 => Either7::Right(Either::Right(Either::Right(Either::Right(Either::Right(Either::Left(t!(i18n, status_draft))))))),
                            _ => Either7::Right(Either::Right(Either::Right(Either::Right(Either::Right(Either::Right(t!(i18n, status_deleted))))))),
                        }}
                    </button>
                }
            }).collect::<Vec<_>>()}
            </div>
            {move || feedback.get().map(|msg| view! {
                <p class="text-green-500 text-xs mt-1">{msg}</p>
            })}
        </div>
    }
}

// ── 仪表盘 ──────────────────────────────────────────────────────────────

#[component]
pub fn AdminPage() -> impl IntoView {
    let i18n = use_i18n();
    let status_res = Resource::new(|| (), |_| get_my_status());

    view! {
        <Title text=move || page_title!(i18n, admin_dashboard)/>
        <Nav/>
        <main class=MAIN>
            <Suspense fallback=move || view! {
                <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
            }>
                {move || match status_res.get() {
                    Some(Err(_)) | None => Either3::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500 mb-4">{move || t!(i18n, admin_access_denied)}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        </div>
                    }),
                    Some(Ok(Some(s))) if s >= 6 => Either3::Right(Either::Left(view! {
                        <h1 class=H1>{move || t!(i18n, admin_dashboard)}</h1>
                        <div class=GRID_2>
                            <LocaleA
                                href="/admin/footballs"
                                class=format!("card p-6 block {} {}", NO_UNDERLINE, HOVER_SHADOW)
                            >
                                <h2 class="text-lg font-semibold text-blue-600 mb-2">
                                    "⚽ " {move || t!(i18n, admin_data)}
                                </h2>
                                <p class="text-sm text-gray-500">{move || t!(i18n, admin_data_desc)}</p>
                            </LocaleA>
                            <LocaleA
                                href="/admin/users"
                                class=format!("card p-6 block {} {}", NO_UNDERLINE, HOVER_SHADOW)
                            >
                                <h2 class="text-lg font-semibold text-blue-600 mb-2">
                                    "👥 " {move || t!(i18n, admin_users)}
                                </h2>
                                <p class="text-sm text-gray-500">{move || t!(i18n, admin_users_desc)}</p>
                            </LocaleA>
                        </div>
                    })),
                    _ => Either3::Right(Either::Right(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">{move || t!(i18n, admin_access_denied)}</p>
                        </div>
                    })),
                }}
            </Suspense>
        </main>
        <Footer/>
    }
}

// ── 用户管理列表 ────────────────────────────────────────────────────────

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let status_res = Resource::new(|| (), |_| get_my_status());
    let query = use_query_map();
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };

    let data = Resource::new_blocking(move || from(), |f| async move { get_admin_users(f).await });
    let update_action = ServerAction::<AdminUpdateUserStatus>::new();

    view! {
        <Title text=move || page_title!(i18n, admin_users)/>
        <Nav/>
        <main class="max-w-7xl mx-auto px-4 py-8">
            <Suspense fallback=move || view! {
                <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
            }>
                {move || match status_res.get() {
                    Some(Err(_)) | None => Either3::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500 mb-4">{move || t!(i18n, admin_access_denied)}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        </div>
                    }),
                    Some(Ok(Some(s))) if s >= 6 => Either3::Right(Either::Left(view! {
                        <div class="text-sm text-gray-500 mb-4 flex items-center gap-1">
                            <LocaleA href="/admin" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_dashboard)}</LocaleA>
                            <span class="text-gray-300">"»"</span>
                            <span class="text-gray-800 dark:text-gray-200 font-medium">{move || t!(i18n, admin_users)}</span>
                        </div>
                        <h1 class=H1>{move || t!(i18n, admin_users)}</h1>

                        <Suspense fallback=move || view! {
                    <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
                }>
                    {move || data.get().map(|result| match result {
                        Err(e) => Either::Left(view! {
                            <p class="text-red-500 text-center">{e.to_string()}</p>
                        }),
                        Ok(d) => {
                            let pi = d.page_info.clone();
                            Either::Right(view! {
                                <div class=format!("{} mb-8", GRID_3)>
                                    {d.items.into_iter().map(|user| {
                                        let username = user.username.clone();
                                        let uid = user.id.clone();
                                        let updated = user.updated_at.clone();
                                        let status = user.status;
                                        let keywords = user.keywords.clone();
                                        let initial = username.chars().next().unwrap_or('?');
                                        view! {
                                            <div class="card p-4 hover:shadow-md transition-shadow">
                                                // 用户名 + 状态标签
                                                <div class="flex items-center gap-2 mb-2">
                                                    <div class="w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-lg shrink-0">
                                                        {initial.to_string()}
                                                    </div>
                                                    <div class="min-w-0 flex-1">
                                                        <div class="flex items-center gap-2 flex-wrap">
                                                            <LocaleA
                                                                href=format!("/admin/users/{}", username.clone())
                                                                class="font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 text-sm no-underline hover:underline"
                                                            >
                                                            {username}
                                                            </LocaleA>
                                                            <UserStatusBadge status/>
                                                        </div>
                                                        <p class="text-xs text-gray-400 mt-0.5">{move || t!(i18n, profile_updated)} {updated}</p>
                                                    </div>
                                                </div>

                                                // 关键词
                                                {if !keywords.is_empty() {
                                                    Either::Left(view! {
                                                        <div class="flex flex-wrap gap-1 mb-2">
                                                            {keywords.iter().take(6).map(|t| {
                                                                let kid = crate::shared::common::record_key(&t.id).to_string();
                                                                let path = format!("/footballs?topic={}", kid);
                                                                let name = t.name.clone();
                                                                view! {
                                                                    <LocaleA href=path class="badge-blue no-underline text-xs">{name}</LocaleA>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    })
                                                } else {
                                                    Either::Right(())
                                                }}

                                                // 操作按钮
                                                <div class="mt-2">
                                                    <UserStatusButtons id=uid initial_status=status action=update_action/>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <Pagination page_info=pi base_url=format!("/{}/admin/users", loc_str.get())/>
                            })
                        }
                    })}>
                </Suspense>
                    })),
                    _ => Either3::Right(Either::Right(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">{move || t!(i18n, admin_access_denied)}</p>
                        </div>
                    })),
                }}
            </Suspense>
        </main>
        <Footer/>
    }
}

// ── 用户管理详情 ────────────────────────────────────────────────────────

#[component]
pub fn AdminUserDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let status_res = Resource::new(|| (), |_| get_my_status());
    let params = use_params_map();
    let username = move || params.read().get("username").unwrap_or_default();

    let data = Resource::new_blocking(
        move || username(),
        |u| async move { admin_get_user(u).await },
    );
    let update_action = ServerAction::<AdminUpdateUserStatus>::new();

    view! {
        <Title text=move || page_title!(i18n, admin_users)/>
        <Nav/>
        <main class=MAIN>
            <Suspense fallback=move || view! {
                <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
            }>
                {move || match status_res.get() {
                    Some(Err(_)) | None => Either3::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500 mb-4">{move || t!(i18n, admin_access_denied)}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        </div>
                    }),
                    Some(Ok(Some(s))) if s >= 6 => Either3::Right(Either::Left(view! {
                <Suspense fallback=move || view! {
                    <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
                }>
                    {move || data.get().map(|result| match result {
                        Err(e) => Either3::<_, _, _>::Left(view! {
                            <p class="text-red-500 text-center">{e.to_string()}</p>
                        }),
                        Ok(None) => Either3::Right(Either::Left(view! {
                            <div class=EMPTY>
                                <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                                <LocaleA href="/admin/users" class="btn-primary">{move || t!(i18n, go_list)}</LocaleA>
                            </div>
                        })),
                        Ok(Some(user)) => {
                            let uid = user.id.clone();
                            let uname = user.username.clone();
                            let email = user.email.clone();
                            let created = user.created_at.clone();
                            let updated = user.updated_at.clone();
                            let status = user.status;
                            let intro_html = user.introduction_html.clone();
                            let keywords = user.keywords.clone();
                            let topics = user.topics.clone();
                            let initial = uname.chars().next().unwrap_or('?');
                            Either3::Right(Either::Right(view! {
                                // 面包屑
                                <div class="text-sm text-gray-500 mb-4 flex items-center gap-1">
                                    <LocaleA href="/admin" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_dashboard)}</LocaleA>
                                    <span class="text-gray-300">"»"</span>
                                    <LocaleA href="/admin/users" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_users)}</LocaleA>
                                    <span class="text-gray-300">"»"</span>
                                    <span class="text-gray-800 dark:text-gray-200 font-medium">{uname.clone()}</span>
                                </div>

                                // 用户信息头部 + 状态操作
                                <div class="card p-6 mb-6">
                                    <div class="flex items-start gap-3">
                                        <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 dark:text-blue-300 font-bold text-2xl shrink-0 mt-1">
                                            {initial.to_string()}
                                        </div>
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">
                                                    {uname}
                                                </h1>
                                                <UserStatusBadge status/>
                                            </div>
                                            <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, registration_time)} {created}</p>
                                            <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)} {updated}</p>
                                            <p class="text-xs text-gray-400 mt-1">Email: {email}</p>
                                        </div>
                                    </div>

                                    // 状态操作按钮
                                    <div class="mt-4">
                                        <UserStatusButtons id=uid initial_status=status action=update_action/>
                                    </div>
                                </div>

                                <UserIntro intro_html=intro_html/>
                                <UserTopics keywords=keywords topics=topics/>
                            }))
                        }
                    })}
                </Suspense>
                    })),
                    _ => Either3::Right(Either::Right(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">{move || t!(i18n, admin_access_denied)}</p>
                        </div>
                    })),
                }}
            </Suspense>
        </main>
        <Footer/>
    }
}

// ── 赛事管理列表 ────────────────────────────────────────────────────────

#[component]
pub fn AdminFootballsPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let status_res = Resource::new(|| (), |_| get_my_status());
    let query = use_query_map();
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };

    let data = Resource::new_blocking(
        move || from(),
        |f| async move { get_admin_footballs(f).await },
    );
    let update_action = ServerAction::<AdminUpdateStatus>::new();

    view! {
        <Title text=move || page_title!(i18n, admin_footballs)/>
        <Nav/>
        <main class="max-w-5xl mx-auto px-4 py-8">
            <Suspense fallback=move || view! {
                <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
            }>
                {move || match status_res.get() {
                    Some(Err(_)) | None => Either3::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500 mb-4">{move || t!(i18n, admin_access_denied)}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        </div>
                    }),
                    Some(Ok(Some(s))) if s >= 6 => Either3::Right(Either::Left(view! {
                <div class="text-sm text-gray-500 mb-4 flex items-center gap-1">
                    <LocaleA href="/admin" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_dashboard)}</LocaleA>
                    <span class="text-gray-300">"»"</span>
                    <span class="text-gray-800 dark:text-gray-200 font-medium">{move || t!(i18n, admin_footballs)}</span>
                </div>
                <h1 class=H1>{move || t!(i18n, admin_footballs)}</h1>

                <Suspense fallback=move || view! {
                    <p class="text-gray-400 text-center py-8">{move || t!(i18n, loading)}</p>
                }>
                    {move || data.get().map(|result| match result {
                        Err(e) => Either3::<_, _, ()>::Left(view! {
                            <p class="text-red-500">{e.to_string()}</p>
                        }),
                        Ok(d) => {
                            let pi = d.page_info.clone();
                            Either3::Right(Either::Left(view! {
                                <div class="space-y-3 mb-8">
                                    {d.items.into_iter().map(|football| {
                                        let Football { id, season, kick_off_at_mdhm8, status, home_team, away_team, ana_type, article_title, .. } = football;
                                        let url = format!("/{}/admin/footballs/{}", loc_str.get(), crate::shared::common::record_key(&id));
                                        let title = if ana_type == 0 {
                                            article_title.unwrap_or_else(|| format!("{} vs {}", home_team, away_team))
                                        } else {
                                            format!("{} vs {}", home_team, away_team)
                                        };
                                        view! {
                                            <div class="card p-4 flex items-center gap-4 flex-wrap">
                                                <div class="flex-1 min-w-0">
                                                    <a
                                                        href=url
                                                        class=format!("font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 {} text-sm", NO_UNDERLINE)
                                                    >
                                                        {title}
                                                    </a>
                                                    <p class="text-xs text-gray-400 mt-1">
                                                        {season} " · " {kick_off_at_mdhm8}
                                                    </p>
                                                </div>
                                                // 状态操作按钮
                                                    <FootballStatusButtons id=id.to_string() initial_status=status action=update_action/>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <Pagination page_info=pi base_url=format!("/{}/admin/footballs", loc_str.get())/>
                            }))
                        }
                    })}
                </Suspense>
                    })),
                    _ => Either3::Right(Either::Right(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">{move || t!(i18n, admin_access_denied)}</p>
                        </div>
                    })),
                }}
            </Suspense>
        </main>
        <Footer/>
    }
}

// ── 赛事管理详情 ────────────────────────────────────────────────────────

#[component]
pub fn AdminFootballDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let status_res = Resource::new(|| (), |_| get_my_status());
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let football_res =
        Resource::new_blocking(move || id(), |i| async move { admin_get_football(i).await });
    let update_action = ServerAction::<AdminUpdateStatus>::new();

    view! {
        <Title text="BiPou"/>
        <main class=MAIN>
            <Suspense fallback=move || view! {
                <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
            }>
                {move || match status_res.get() {
                    Some(Err(_)) | None => Either3::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500 mb-4">{move || t!(i18n, admin_access_denied)}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        </div>
                    }),
                    Some(Ok(Some(s))) if s >= 6 => Either3::Right(Either::Left(view! {
                <Suspense fallback=move || view! {
                    <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div>
                }>
                    {move || football_res.get().map(|fr| match fr {
                        Err(e) => Either3::Left(view! {
                            <p class="text-red-500 text-center py-8">{e.to_string()}</p>
                        }),
                        Ok(None) => Either3::Right(Either::Left(view! {
                            <div class=EMPTY>
                                <p class=NO_DATA>{move || t!(i18n, no_data)}</p>
                            </div>
                        })),
                        Ok(Some(f)) => {
                            let fid = f.id.clone();
                            let status = f.status;
                            let title = if f.ana_type == 0 {
                                f.article_title.clone().unwrap_or_else(|| f.title())
                            } else {
                                f.title()
                            };
                            Either3::Right(Either::Right(view! {
                                // 面包屑
                                <div class="text-sm text-gray-500 mb-4 flex items-center gap-1">
                                    <LocaleA href="/admin" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_dashboard)}</LocaleA>
                                    <span class="text-gray-300">"»"</span>
                                    <LocaleA href="/admin/footballs" class="text-gray-500 hover:text-blue-600 no-underline">{move || t!(i18n, admin_footballs)}</LocaleA>
                                    <span class="text-gray-300">"»"</span>
                                    <span class="text-gray-800 dark:text-gray-200 font-medium">{title}</span>
                                </div>

                                // 状态操作按钮
                                <div class="card p-4 mb-4">
                                    <FootballStatusButtons id=fid initial_status=status action=update_action/>
                                </div>

                                // 公开详情
                                <crate::pages::football::FootballDetailPage/>
                            }))
                        }
                    })}
                </Suspense>
                    })),
                    _ => Either3::Right(Either::Right(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">{move || t!(i18n, admin_access_denied)}</p>
                        </div>
                    })),
                }}
            </Suspense>
        </main>
    }
}
