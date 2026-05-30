use crate::i18n::{t, t_display, use_i18n};
use crate::page_title;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{CaptchaCore, CaptchaState, MarkdownEditor, TopicInput};
use crate::shared::common::Either3;
use crate::shared::constant::{GRID_2, H1, HOVER_UNDERLINE, TEXT_SUBTLE};
use crate::shared::locale::LocaleA;
use leptos::either::Either;

// ── Server functions ──────────────────────────────────────────────────────────

/// Sign in
#[server]
pub async fn sign_in(
    signature: String,
    password: String,
    captcha_token: String,
    captcha_answer: String,
) -> Result<(), ServerFnError> {
    use crate::server::{auth as auth_mod, captcha, user_db};
    use axum::http::{HeaderValue, header};

    if captcha::verify_token(&captcha_token, &captcha_answer).is_none() {
        return Err(ServerFnError::new("captcha_invalid"));
    }

    let auth_user = user_db::sign_in(&signature, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let cookie = auth_mod::make_set_cookie("fs_token", &auth_user.token, 30 * 24 * 3600);
    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.insert_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(())
}

// ── Sign Out server function ──────────────────────────────────────────────────

#[server]
pub async fn sign_out() -> Result<(), ServerFnError> {
    use crate::server::auth as auth_mod;
    use axum::http::{HeaderValue, header};

    let cookie = auth_mod::make_clear_cookie("fs_token");
    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.insert_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(())
}

// ── Register server function ──────────────────────────────────────────────────

#[server]
pub async fn register(
    username: String,
    email: String,
    password: String,
    confirm_password: String,
    introduction: String,
    topics: String,
    lang: String,
    captcha_token: String,
    captcha_answer: String,
) -> Result<String, ServerFnError> {
    use crate::server::upload::move_uploads;
    use crate::server::{captcha, email as email_mod, user_db};
    use crate::shared::common::{into_rid, record_key};

    if captcha::verify_token(&captcha_token, &captcha_answer).is_none() {
        return Err(ServerFnError::new("captcha_invalid"));
    }

    if password != confirm_password {
        return Err(ServerFnError::new("register_password_mismatch"));
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_upper || !has_lower || !has_digit {
        return Err(ServerFnError::new("register_password_weak"));
    }

    // 将临时图片持久化到 /uploads/user/active/imgs/
    let introduction = move_uploads(&introduction, "user", false)?;

    let data = user_db::RegisterData {
        username,
        email,
        password,
        introduction,
        topics,
    };

    let (user_id, username) = user_db::register_user(data)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_rid = into_rid(&user_id, "users");
    if let Ok(Some((email_addr, _))) = user_db::get_user_email_username(&user_rid).await {
        let kid = record_key(&user_id).to_string();
        let uname = username.clone();
        // 邮件发送与注册解耦，避免 SMTP 阻塞响应
        tokio::spawn(async move {
            let _ = email_mod::send_activation_email(&lang, &uname, &kid, &email_addr).await;
        });
    }

    Ok(username)
}

// ── Activate / Resend server functions ────────────────────────────────────────

#[server]
pub async fn activate_user(user_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::user_db;
    use crate::shared::common::into_rid;
    let rid = into_rid(&user_id, "users");
    user_db::activate_user(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn resend_activation(user_id: String, lang: String) -> Result<(), ServerFnError> {
    use crate::server::{email as email_mod, user_db};
    use crate::shared::common::into_rid;
    let rid = into_rid(&user_id, "users");
    let (email, username) = user_db::get_user_email_username(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("user_not_found".to_string()))?;
    email_mod::send_activation_email(&lang, &username, &user_id, &email)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Captcha server function ────────────────────────────────────────────────────

#[server]
pub async fn get_captcha() -> Result<(String, String, u8), ServerFnError> {
    use crate::server::captcha;
    let c = captcha::generate_captcha();
    Ok((c.svg, c.token, c.answer))
}

// ── Sign In page component ────────────────────────────────────────────────────

#[component]
pub fn SignInPage() -> impl IntoView {
    let i18n = use_i18n();
    let loc = i18n.get_locale().to_string();
    let action = ServerAction::<SignIn>::new();
    let navigate = leptos_router::hooks::use_navigate();
    let auth_res = use_context::<crate::app::AuthResource>();

    Effect::new(move |_| {
        if let Some(Ok(())) = action.value().get() {
            if let Some(ref res) = auth_res {
                res.refetch();
            }
            navigate(&["/", &loc, "/footballs"].join(""), Default::default());
        }
    });

    let cap_state = CaptchaState::new();
    provide_context(cap_state.clone());

    // 提交出错后刷新验证码
    Effect::new(move |_| {
        if let Some(Err(_)) = action.value().get() {
            cap_state.refresh_trigger.update(|n| *n += 1);
        }
    });

    let btn = Signal::derive(move || t_display!(i18n, sign_in).to_string());
    let pending = Signal::derive(move || t_display!(i18n, signing_in).to_string());

    view! {
        <Title text=move || page_title!(i18n, user_sign_in)/>
        <main class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="card p-8 w-full max-w-sm">
                <h1 class=[H1, "text-center"].join(" ")>
                    {move || t!(i18n, user_sign_in)}
                </h1>

                <div class="space-y-4">
                    <ActionForm action=action>
                        <div>
                            <label class="form-label">{move || t!(i18n, sign_in_account)}</label>
                            <input type="text" name="signature" required
                                   class="form-input " autocomplete="username"/>
                        </div>
                        <div>
                            <label class="form-label">{move || t!(i18n, sign_in_password)}</label>
                            <input type="password" name="password" required
                                   class="form-input " autocomplete="current-password"/>
                        </div>

                        <CaptchaCore/>

                        <button type="submit"
                            disabled=move || !cap_state.ok.get() || action.pending().get()
                            class=move || if cap_state.ok.get() {
                                "btn-primary w-full justify-center mt-4".to_string()
                            } else {
                                "w-full justify-center bg-gray-300 text-gray-500 rounded-lg py-2 px-4 cursor-not-allowed mt-4".to_string()
                            }
                        >
                            {move || if action.pending().get() { pending.get() } else { btn.get() }}
                        </button>

                        // Error
                        {move || action.value().get().and_then(|r| r.err()).map(|e| {
                            let raw = e.to_string();
                            let msg = if raw.contains("captcha_invalid") {
                                t_display!(i18n, captcha_invalid).to_string()
                            } else if raw.contains("sign_in_incorrect") {
                                t_display!(i18n, sign_in_incorrect).to_string()
                            } else if raw.contains("sign_in_not_activation") {
                                t_display!(i18n, sign_in_not_activation).to_string()
                            } else if raw.contains("sign_in_banned") {
                                t_display!(i18n, sign_in_banned).to_string()
                            } else if raw.contains("sign_in_security_problem") {
                                t_display!(i18n, sign_in_security_problem).to_string()
                            } else {
                                raw
                            };
                            view! { <p class="text-red-500 text-sm text-center">{msg}</p> }
                        })}
                    </ActionForm>
                </div>

                <p class="mt-4 text-sm text-center text-gray-500">
                    {move || t!(i18n, sign_in_new_user)} " "
                    <LocaleA href="/register" class=["text-blue-500", HOVER_UNDERLINE].join(" ")>{move || t!(i18n, sign_in_create_account)}</LocaleA>
                </p>
            </div>
        </main>
    }
}

// ── Sign Out page component ───────────────────────────────────────────────────

#[component]
pub fn SignOutPage() -> impl IntoView {
    let i18n = use_i18n();
    let action = ServerAction::<SignOut>::new();
    let navigate = leptos_router::hooks::use_navigate();
    let auth_res = use_context::<crate::app::AuthResource>();

    let loc = i18n.get_locale().to_string();

    Effect::new(move |_| {
        action.dispatch(SignOut {});
    });
    Effect::new(move |_| {
        if action.value().get().is_some() {
            if let Some(ref res) = auth_res {
                res.refetch();
            }
            navigate(&["/", &loc, "/"].join(""), Default::default());
        }
    });

    view! {
        <div class="min-h-screen flex items-center justify-center">
            <p class=[TEXT_SUBTLE, "text-lg"].join(" ")>{move || t!(i18n, signing_out)}</p>
        </div>
    }
}

// ── Register page component ───────────────────────────────────────────────────

#[component]
pub fn RegisterPage() -> impl IntoView {
    let i18n = use_i18n();
    let action = ServerAction::<Register>::new();
    let (success, set_success) = signal(false);
    let (reg_username, set_reg_username) = signal(String::new());

    Effect::new(move |_| {
        if let Some(Ok(name)) = action.value().get() {
            set_reg_username.set(name);
            set_success.set(true);
        }
    });

    let cap_state = CaptchaState::new();
    provide_context(cap_state.clone());

    // 提交出错后刷新验证码
    Effect::new(move |_| {
        if let Some(Err(_)) = action.value().get() {
            cap_state.refresh_trigger.update(|n| *n += 1);
        }
    });

    let btn = Signal::derive(move || t_display!(i18n, register).to_string());
    let pending_btn = Signal::derive(move || t_display!(i18n, submitting).to_string());

    view! {
        <Title text=move || page_title!(i18n, user_register)/>
        <main class="max-w-2xl mx-auto px-4 py-8">
            // 表单卡片：relative 让弹框相对于此定位
            <div class="card p-8 relative">
                <h1 class=H1>
                    {move || t!(i18n, user_register)}
                </h1>

                // 表单始终渲染，成功时模糊 + 禁止交互
                <div style:opacity=move || if success.get() { "0.35" } else { "1" }
                     style:filter=move || if success.get() { "blur(4px)" } else { "none" }
                     style:pointer-events=move || if success.get() { "none" } else { "auto" }
                     style:transition="all 0.3s"
                >
                    <ActionForm action=action>
                        <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                        <div class=GRID_2>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_username)} " *"</label>
                                <input type="text" name="username" required
                                       class="form-input " pattern="[a-z0-9_-]+" autocomplete="username"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_email)} " *"</label>
                                <input type="email" name="email" required
                                       class="form-input " autocomplete="email"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_password)} " *"</label>
                                <input type="password" name="password" required
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_confirm_password)} " *"</label>
                                <input type="password" name="confirm_password" required
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                        </div>
                        <div class="space-y-4 mt-4">
                        <div>
                            <TopicInput/>
                        </div>
                        <div>
                            <label class="form-label">
                                {move || t!(i18n, register_intro)}
                            </label>
                            <MarkdownEditor name="introduction" rows=4 value=RwSignal::new("## About Me\n我关注足球数据与计算。".to_string()) />
                        </div>
                        </div>
                        <CaptchaCore/>

                        <button type="submit"
                            disabled=move || !cap_state.ok.get() || action.pending().get()
                            class=move || if cap_state.ok.get() {
                                "btn-primary w-full justify-center mt-4".to_string()
                            } else {
                                "w-full justify-center bg-gray-300 text-gray-500 rounded-lg py-2 px-4 cursor-not-allowed mt-4".to_string()
                            }
                        >
                            {move || if action.pending().get() { pending_btn.get() } else { btn.get() }}
                        </button>

                        // Error
                        {move || action.value().get().and_then(|r| r.err()).map(|e| {
                            let raw = e.to_string();
                            let msg = if raw.contains("captcha_invalid") {
                                t_display!(i18n, captcha_invalid).to_string()
                            } else if raw.contains("register_password_mismatch") {
                                t_display!(i18n, register_password_mismatch).to_string()
                            } else if raw.contains("register_password_weak") {
                                t_display!(i18n, register_password_weak).to_string()
                            } else if raw.contains("register_exist") {
                                t_display!(i18n, register_exist).to_string()
                            } else if raw.contains("upload_failed") {
                                t_display!(i18n, upload_failed).to_string()
                            } else {
                                raw
                            };
                            view! { <p class="text-red-500 text-sm text-center">{msg}</p> }
                        })}
                    </ActionForm>
                </div>

                <p class="mt-4 text-sm text-center text-gray-500">
                    {move || t!(i18n, register_have_account)} " "
                    <LocaleA href="/sign-in" class=["text-blue-500", HOVER_UNDERLINE].join(" ")>{move || t!(i18n, register_go_sign_in)}</LocaleA>
                </p>

                // 成功弹框
                <Show when=move || success.get() fallback=|| ()>
                    <div class="modal-overlay">
                        <div class="modal-card">
                            <div class="modal-icon">"✓"</div>
                            <p class="modal-text">
                                {move || {
                                    let name = reg_username.get();
                                    t_display!(i18n, register_success, username = &name).to_string()
                                }}
                            </p>
                            <div class="modal-actions">
                                <a href=move || ["/", &i18n.get_locale().to_string(), "/sign-in"].join("") class="btn-primary modal-btn">
                                                                    {move || t!(i18n, register_go_sign_in)}
                                                                </a>
                                                                <a href=move || ["/", &i18n.get_locale().to_string(), "/"].join("") class="modal-btn-primary">
                                    {move || t!(i18n, go_home)}
                                </a>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </main>
    }
}

// ── Activation page component ─────────────────────────────────────────────────

#[component]
pub fn UserActivatePage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let user_id = move || params.read().get("id").unwrap_or_default();

    let activate_res = Resource::new_blocking(
        move || user_id(),
        |id| async move { activate_user(id).await },
    );

    let resend_action = ServerAction::<ResendActivation>::new();
    let (resent, set_resent) = signal(false);

    Effect::new(move |_| {
        if let Some(Ok(())) = resend_action.value().get() {
            set_resent.set(true);
        }
    });

    view! {
        <Title text=move || page_title!(i18n, user_activate)/>
        <main class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="card p-8 text-center max-w-md">
                <Suspense fallback=move || view! { <p class="text-gray-400">{move || t!(i18n, loading)}</p> }>
                    {move || activate_res.get().map(move |result| match result {
                        Err(e) => Either3::Left(view! {
                            <p class="text-red-500">{e.to_string()}</p>
                        }),
                        Ok(Some(username)) => Either3::Right(Either::Left(view! {
                            <h1 class="text-xl font-bold text-green-600 mb-4">
                                {move || t!(i18n, user_activated)}
                            </h1>
                            <p class="text-gray-600 mb-4">{username}</p>
                            <LocaleA href="/sign-in" class="btn-primary">{move || t!(i18n, sign_in)}</LocaleA>
                        })),
                        Ok(None) => Either3::Right(Either::Right(view! {
                            <Show when=move || !resent.get() fallback=move || view! {
                                <p class="text-green-600 font-semibold mb-4">{move || t!(i18n, user_re_activate)}</p>
                            }>
                                <p class="text-gray-500 mb-4">{move || t!(i18n, user_activate_problem)}</p>
                                <ActionForm action=resend_action>
                                    <input type="hidden" name="user_id" value=user_id/>
                                    <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                                    <button type="submit" class="btn-primary">
                                        {move || t!(i18n, resend_activation)}
                                    </button>
                                </ActionForm>
                            </Show>
                        })),
                    })}
                </Suspense>
            </div>
        </main>
    }
}
