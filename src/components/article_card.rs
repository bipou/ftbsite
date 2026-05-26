use crate::i18n::{t, use_i18n};
use crate::models::Football;
use crate::shared::constant::{FLEX_BETWEEN, HOVER_SHADOW, TEXT_SUBTLE, TEXT_XS_MUTED};
use crate::shared::locale::LocaleA;
use leptos::either::Either;
use leptos::prelude::*;

/// 用户 / AI 分析文章卡片
#[component]
pub fn ArticleCard(football: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title = football.article_title.unwrap_or_else(|| football.home_team);
    let summary = football.summary;
    let created = football.created_at;
    let is_ai = football.ana_type > 0;
    let detail_path = format!(
        "/footballs/{}",
        crate::shared::common::record_key(&football.id)
    );

    view! {
        <div class=format!("card p-4 {} {}", HOVER_SHADOW, "fc-status-1")>
            <div class=format!("{} mb-2", FLEX_BETWEEN)>
                <LocaleA
                    href=detail_path
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 no-underline text-lg leading-tight truncate"
                >
                    {title}
                </LocaleA>
                <span class="text-sm text-gray-400 ml-2 whitespace-nowrap">
                    {if is_ai { "AI" } else { "" }}
                </span>
            </div>

            <div class=format!("text-sm {} mb-3", TEXT_SUBTLE)>
                <span class="text-blue-500">{created}</span>
            </div>

            {if let Some(s) = summary {
                Either::Left(view! {
                    <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mt-2">{s}</p>
                })
            } else {
                Either::Right(())
            }}

            <div class=format!("{} mt-3", FLEX_BETWEEN)>
                <span class=format!("text-sm {}", TEXT_XS_MUTED)>
                    // TODO: 查用户名
                    "球迷"
                </span>
                <span class="text-sm text-gray-400">
                    {move || t!(i18n, football_hits)} " " {football.hits}
                </span>
            </div>
        </div>
    }
}
