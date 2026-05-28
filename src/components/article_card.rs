use crate::components::football_card::{status_badge, status_class};
use crate::i18n::{t, t_display, use_i18n};
use crate::models::Football;
use crate::shared::constant::{CARD_TITLE, FLEX_BETWEEN, HOVER_SHADOW, TEXT_SUBTLE, TEXT_XS_MUTED};
use crate::shared::fns::get_username_by_id;
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

    let status = football.status;
    let card_class = format!("card p-4 {} {}", HOVER_SHADOW, status_class(status));
    let badge = status_badge(status);
    let badge_or_label = if !badge.is_empty() {
        badge.to_string()
    } else if is_ai {
        t_display!(i18n, analysis_ai).to_string()
    } else {
        String::new()
    };

    // 用户文章：获取作者 user_id
    let user_id = if football.ana_type == 0 {
        football.analyses.first().and_then(|a| a.user_id.clone())
    } else {
        None
    };

    let author_name = Resource::new(
        move || user_id.clone(),
        |uid| async move {
            match uid {
                Some(id) if !id.is_empty() => get_username_by_id(id).await.ok().flatten(),
                _ => None,
            }
        },
    );

    view! {
        <div class=card_class>
            <div class=format!("{} mb-2", FLEX_BETWEEN)>
                <LocaleA
                    href=detail_path
                    target="_blank"
                    rel="noopener noreferrer"
                    class=CARD_TITLE
                >
                    {title}
                </LocaleA>
                <span class="text-sm ml-2 whitespace-nowrap">
                    {badge_or_label}
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
                    {move || author_name.get().flatten().unwrap_or_default()}
                </span>
                <span class="text-sm text-gray-400">
                    {move || t!(i18n, football_hits)}{move || t!(i18n, colon)} {football.hits}
                </span>
            </div>
        </div>
    }
}
