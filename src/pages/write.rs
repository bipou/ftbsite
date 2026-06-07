#[cfg(feature = "hydrate")]
use crate::app::{AuthMode, AuthPanelSignal};
use crate::i18n::{t, t_display, use_i18n};
use crate::page_title;
use crate::server_error_text;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::{CaptchaCore, CaptchaState, CategorySelect, MarkdownEditor, TopicInput};
use crate::shared::constant::{H1, WIDE};
use crate::shared::locale::LocaleA;

#[server]
pub async fn get_all_categories() -> Result<Vec<crate::models::Category>, ServerFnError> {
    use crate::server::category_db;
    let mut cats = category_db::get_categories()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    // pinned=true 最前，无字段次之，pinned=false 隐藏；各自 level ASC
    cats.retain(|c| c.pinned != Some(false));
    cats.sort_by(|a, b| {
        fn rank(p: Option<bool>) -> u8 {
            match p {
                Some(true) => 0,
                None => 1,
                Some(false) => 2,
            }
        }
        a.level
            .cmp(&b.level)
            .then(rank(a.pinned).cmp(&rank(b.pinned)))
    });
    Ok(cats)
}

#[server]
pub async fn post_analysis(
    title: String,
    category_id: String,
    summary: String,
    topic_csv: String,
    content: String,
    status: i8,
    captcha_token: String,
    captcha_answer: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{decode_jwt, get_cookie_value};
    use crate::server::{analysis_db, captcha, football_db, topic_db, user_db};
    use axum::http::HeaderMap;

    // 草稿无需验证码
    if status != -1 {
        if captcha::verify_token(&captcha_token, &captcha_answer).is_none() {
            return Err(ServerFnError::new("captcha_invalid"));
        }
    }

    // 必填校验（除话题外均为必填）
    if status != -1 {
        if title.trim().is_empty() {
            return Err(ServerFnError::new("title_required"));
        }
        if category_id.trim().is_empty() {
            return Err(ServerFnError::new("category_required"));
        }
        if summary.trim().is_empty() {
            return Err(ServerFnError::new("summary_required"));
        }
        if content.trim().is_empty() {
            return Err(ServerFnError::new("content_required"));
        }
    }

    // 摘要限 200 字符
    if summary.chars().count() > 200 {
        return Err(ServerFnError::new("summary_too_long"));
    }

    let headers: HeaderMap = leptos_axum::extract().await?;
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let token =
        get_cookie_value(cookie, "fs_token").ok_or_else(|| ServerFnError::new("not_sign_in"))?;
    let claims = decode_jwt(&token).map_err(|e| ServerFnError::new(e.to_string()))?;
    let user = user_db::get_user_by_username(&claims.username)
        .await
        .map_err(|e| ServerFnError::new(e))?
        .ok_or_else(|| ServerFnError::new("user_not_found"))?;

    let fid = football_db::insert_article(&title, &category_id, status)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // 将临时图片持久化到 /uploads/football/draft/ 或 /uploads/football/active/
    let content = crate::server::upload::move_uploads(&content, "football", status == -1)
        .map_err(|e| ServerFnError::new(e))?;

    analysis_db::insert_analysis(&fid, &content, &user.id, &summary)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    if !topic_csv.is_empty() {
        let tids = topic_db::create_topics_from_names(&topic_csv)
            .await
            .map_err(|e| ServerFnError::new(e))?;
        topic_db::link_topics_to_football(&fid, &tids)
            .await
            .map_err(|e| ServerFnError::new(e))?;
    }

    Ok(title)
}

// ── Category selector ────────────────────────────────────────────────────────

#[component]
fn CategorySection(selected: RwSignal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let cats_res = Resource::new(|| (), |_| get_all_categories());
    view! {
        <div class="pr-8">
            <div class="flex">
                <label class="form-label shrink-0">{move || t!(i18n, football_category)}</label>
                <Suspense fallback=|| ()>
                    {move || cats_res.get().map(|r| r.ok()).flatten().map(|all| {
                        view! {
                            <div class="flex flex-wrap gap-2">
                                <CategorySelect all=all selected=selected expandable=true/>
                            </div>
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

// ── Page ─────────────────────────────────────────────────────────────────────

#[component]
pub fn WriteArticlePage() -> impl IntoView {
    let i18n = use_i18n();

    // 登录守卫（仅客户端执行，避免 SSR 时 auth 未就绪误判）
    // 等 auth 资源加载完毕后一次性检查，未登录仅弹出签入面板不跳转
    #[cfg(feature = "hydrate")]
    {
        let auth_panel = use_context::<AuthPanelSignal>();
        let auth_res = use_context::<crate::shared::auth::AuthResource>();
        let guard_done = RwSignal::new(false);
        Effect::new(move |_| {
            if guard_done.get() {
                return;
            }
            // Resource::get() → Option<Result<Option<AuthUser>>>
            // None = 未加载，Some(Ok(None)) = 已加载但未登录
            match auth_res.as_ref().and_then(|r| r.get()) {
                None => return,         // 尚未加载完毕，等待
                Some(Ok(Some(_))) => {} // 已登录，放行
                _ => {
                    // 未登录或出错 → 弹出签入面板
                    if let Some(ref ap) = auth_panel {
                        ap.set(Some(AuthMode::SignIn));
                    }
                }
            }
            guard_done.set(true);
        });
    }

    let title = RwSignal::new(String::new());
    let category_id = RwSignal::new(String::new());
    let summary = RwSignal::new(String::new());
    let topic_csv = RwSignal::new(String::new());
    let content = RwSignal::new("## Share Analysis\n\n在此撰写您的足球分析…".to_string());
    let (success, set_success) = signal(false);
    let (article_title, set_article_title) = signal(String::new());
    let cat_err = RwSignal::new(false);
    let content_err = RwSignal::new(false);

    let post_action = ServerAction::<PostAnalysis>::new();
    let cap_state = CaptchaState::new();
    provide_context(cap_state.clone());

    // 提交出错后刷新验证码
    Effect::new(move |_| {
        if let Some(Err(_)) = post_action.value().get() {
            cap_state.refresh_trigger.update(|n| *n += 1);
        }
    });

    let form_ref = NodeRef::<leptos::html::Form>::new();

    let submit = move |status: i8| {
        if status != -1 {
            if let Some(f) = form_ref.get() {
                if !f.report_validity() {
                    return;
                }
            }
            cat_err.set(category_id.get().trim().is_empty());
            content_err.set(content.get().trim().is_empty());
            if cat_err.get() || content_err.get() {
                return;
            }
        }
        post_action.dispatch(PostAnalysis {
            title: title.get(),
            category_id: category_id.get(),
            summary: summary.get(),
            topic_csv: topic_csv.get(),
            content: content.get(),
            status,
            captcha_token: cap_state.token.get(),
            captcha_answer: cap_state.answer.get(),
        });
    };

    Effect::new(move |_| {
        if let Some(Ok(t)) = post_action.value().get() {
            set_article_title.set(t);
            set_success.set(true);
        }
    });

    let pending = post_action.pending();

    let title_ph = untrack(|| t_display!(i18n, article_title).to_string());

    view! {
        <Title text=move || page_title!(i18n, write_article)/>
        <main class=WIDE>
                    <h1 class=H1>{move || t!(i18n, write_article)}</h1>

            <div style:opacity=move || if success.get() { "0.35" } else { "1" }
                style:filter=move || if success.get() { "blur(4px)" } else { "none" }
                style:pointer-events=move || if success.get() { "none" } else { "auto" }
                style:transition="all 0.3s"
            >
                <div class="space-y-4">
                    <form node_ref=form_ref on:submit=|ev| ev.prevent_default()>
                    <div>
                        <input
                            type="text"
                            class="form-input w-full text-lg font-semibold"
                            placeholder=title_ph
                            required
                            prop:value=title
                            on:input=move |ev| title.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class="form-label">{move || t!(i18n, article_summary)}</label>
                        <textarea
                            rows="3"
                            maxlength="200"
                            required
                            class="form-input w-full"
                            prop:value=summary
                            on:input=move |ev| summary.set(event_target_value(&ev))
                        />
                    </div>

                    <CategorySection selected=category_id/>
                    {move || cat_err.get().then(|| view! { <p class="text-red-500 text-xs mt-1">请选择类别</p> })}

                    <TopicInput csv_out=topic_csv/>

                    <div>
                        <label class="form-label">{move || t!(i18n, analysis_content)}</label>
                        <MarkdownEditor
                            name="content"
                            value=content
                            scope="football"
                            required=true
                        />
                        {move || content_err.get().then(|| view! { <p class="text-red-500 text-xs mt-1">内容不能为空</p> })}
                    </div>

                    <CaptchaCore/>

                    <div class="flex gap-4 pt-4">
                        <button
                            type="button"
                            class="btn-secondary flex-1"
                            disabled=move || pending.get()
                            on:click=move |_| submit(-1)
                        >
                            {move || if pending.get() { t_display!(i18n, submitting).to_string() } else { t_display!(i18n, save_draft).to_string() }}
                        </button>
                        <button
                            type="button"
                            class="btn-primary flex-1"
                            disabled=move || !cap_state.ok.get() || pending.get()
                            on:click=move |_| submit(0)
                        >
                            {move || if pending.get() { t_display!(i18n, submitting).to_string() } else { t_display!(i18n, submit_article).to_string() }}
                        </button>
                    </div>
                    </form>

                    {move || post_action.value().get().and_then(|r| r.err()).map(|e| {
                        let raw = e.to_string();
                        let msg = server_error_text!(i18n, raw,
                            "captcha_invalid" => captcha_invalid,
                            "not_sign_in" => not_sign_in,
                            "user_not_found" => user_not_found,
                            "summary_too_long" => summary_too_long,
                        );
                        view! { <p class="text-red-500 text-sm text-center mt-2">{msg}</p> }
                    })}
                </div>
            </div>

            <Show when=move || success.get() fallback=|| ()>
                <div class="modal-overlay">
                    <div class="modal-card">
                        <div class="modal-icon">"✓"</div>
                        <p class="modal-text">
                            <span class="modal-username">{article_title.get()}</span>
                            <br/>
                            {t_display!(i18n, share_success).to_string()}
                            <br/>
                            {t_display!(i18n, review_notice).to_string()}
                        </p>
                        <div class="modal-actions">
                            <LocaleA href="/footballs" class="btn-primary modal-btn">
                                {move || t!(i18n, go_list)}
                            </LocaleA>
                            <LocaleA href="/" class="modal-btn-primary">
                                {move || t!(i18n, go_home)}
                            </LocaleA>
                        </div>
                    </div>
                </div>
            </Show>
        </main>
    }
}
