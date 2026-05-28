use crate::i18n::{t, t_display, use_i18n};
use crate::page_title;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::{CategorySelect, Footer, MarkdownEditor, Nav, TopicInput};
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
    topic_csv: String,
    content: String,
    status: i8,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{decode_jwt, get_cookie_value};
    use crate::server::{analysis_db, football_db, topic_db, user_db};
    use axum::http::HeaderMap;

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
    let draft = status == -1;
    let content = crate::server::upload::move_uploads(&content, "football", draft)
        .map_err(|e| ServerFnError::new(e))?;

    analysis_db::insert_analysis(&fid, &content, &user.id)
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
    let cats_res = Resource::new(|| (), |_| get_all_categories());
    view! {
        <div class="pr-8">
            <div class="flex">
                <label class="form-label shrink-0">{move || t!(use_i18n(), football_category)}{move || t!(use_i18n(), colon)}</label>
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

    let title = RwSignal::new(String::new());
    let category_id = RwSignal::new(String::new());
    let topic_csv = RwSignal::new(String::new());
    let content = RwSignal::new("## Share Analysis\n\n在此撰写您的足球分析…".to_string());
    let (success, set_success) = signal(false);
    let (article_title, set_article_title) = signal(String::new());

    let post_action = ServerAction::<PostAnalysis>::new();

    let submit = move |status: i8| {
        post_action.dispatch(PostAnalysis {
            title: title.get(),
            category_id: category_id.get(),
            topic_csv: topic_csv.get(),
            content: content.get(),
            status,
        });
    };

    Effect::new(move |_| {
        if let Some(Ok(t)) = post_action.value().get() {
            set_article_title.set(t);
            set_success.set(true);
        }
    });

    let pending = post_action.pending();

    let title_ph = t_display!(i18n, article_title).to_string();

    view! {
        <Title text=move || page_title!(i18n, write_article)/>
        <Nav/>
        <main class=WIDE>
            <h1 class=H1>{move || t!(i18n, write_article)}</h1>

            <div style:opacity=move || if success.get() { "0.35" } else { "1" }
                style:filter=move || if success.get() { "blur(4px)" } else { "none" }
                style:pointer-events=move || if success.get() { "none" } else { "auto" }
                style:transition="all 0.3s"
            >
                <div class="space-y-4">
                    <div>
                        <input
                            type="text"
                            class="form-input w-full text-lg font-semibold"
                            placeholder=title_ph
                            prop:value=title
                            on:input=move |ev| title.set(event_target_value(&ev))
                        />
                    </div>

                    <CategorySection selected=category_id/>

                    <TopicInput csv_out=topic_csv/>

                    <div>
                        <label class="form-label">{move || t!(i18n, analysis_content)}</label>
                        <MarkdownEditor
                            name="content"
                            value=content
                            scope="football"
                        />
                    </div>

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
                            disabled=move || pending.get()
                            on:click=move |_| submit(0)
                        >
                            {move || if pending.get() { t_display!(i18n, submitting).to_string() } else { t_display!(i18n, submit_article).to_string() }}
                        </button>
                    </div>

                    {move || post_action.value().get().and_then(|r| r.err()).map(|e| {
                        let raw = e.to_string();
                        let msg = if raw.contains("not_sign_in") {
                            t_display!(i18n, not_sign_in).to_string()
                        } else if raw.contains("user_not_found") {
                            t_display!(i18n, user_not_found).to_string()
                        } else {
                            raw
                        };
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
        <Footer/>
    }
}
