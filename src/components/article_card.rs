use crate::components::football_card::{status_badge, status_class};
use crate::i18n::{t, t_display, use_i18n};
use crate::models::Football;
use crate::shared::common::Either3;
use crate::shared::constant::{
    BADGE_GRAY, BADGE_GRAY_NO_UL, FLEX_BETWEEN, HOVER_SHADOW, TEXT_SUBTLE, TEXT_XS_MUTED,
};
use crate::shared::fns::get_username_by_id;
use crate::shared::locale::{LocaleA, use_locale};
use leptos::either::Either;
use leptos::prelude::*;

/// 用户 / AI 分析文章卡片
#[component]
pub fn ArticleCard(football: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title = football.article_title.unwrap_or_default();
    let summary = football.summary.map(|s| {
        if s.chars().count() > 80 {
            let mut t: String = s.chars().take(80).collect();
            t.push_str("...");
            t
        } else {
            s
        }
    });
    let updated = football.updated_at;
    let is_ai = football.ana_type > 0;
    let detail_path = [
        "/footballs/",
        &crate::shared::common::record_key(&football.id),
    ]
    .join("");

    let status = football.status;
    let card_class = ["card", "p-4", HOVER_SHADOW, status_class(status), "min-w-0"].join(" ");
    let badge = status_badge(status);
    let badge_or_label = if !badge.is_empty() {
        badge.to_string()
    } else if is_ai {
        t_display!(i18n, analysis_ai).to_string()
    } else {
        String::new()
    };

    // 类别
    let loc_str = use_locale();
    let category = football.category;
    let cat_kid = category
        .as_ref()
        .map(|c| crate::shared::common::record_key(&c.id).to_string());
    let cat_name = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category.as_ref().and_then(|c| c.name.get(&loc).cloned())
    });

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
            <div class=[FLEX_BETWEEN, "mb-2"].join(" ")>
                <LocaleA
                    href=detail_path
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 no-underline text-lg leading-tight min-w-0"
                >
                    {title}
                </LocaleA>
                <span class="text-sm ml-2 whitespace-nowrap">
                    {badge_or_label}
                </span>
            </div>

            <div class=["text-sm", TEXT_SUBTLE, "mb-3", "space-x-2"].join(" ")>
                {move || {
                    let n = cat_name.get();
                    if n.is_none() {
                        Either3::Left(())
                    } else if let Some(kid) = &cat_kid {
                        let href = ["/", &loc_str.get(), "/footballs?category=", kid].join("");
                        Either3::Right(Either::Left(view! {
                            <a href=href class=BADGE_GRAY_NO_UL>{n.unwrap_or_default()}</a>
                        }))
                    } else {
                        Either3::Right(Either::Right(
                            view! { <span class=BADGE_GRAY>{n.unwrap_or_default()}</span> },
                        ))
                    }
                }}
                <span class="text-blue-500">{updated}</span>
            </div>

            {if let Some(s) = summary {
                Either::Left(view! {
                    <p class="text-sm text-gray-600 dark:text-gray-400 my-0">{s}</p>
                })
            } else {
                Either::Right(())
            }}

            <div class=[FLEX_BETWEEN, "mt-3"].join(" ")>
                <span class=["text-sm", TEXT_XS_MUTED].join(" ")>
                    <Suspense fallback=|| ()>
                        {move || author_name.get().flatten().unwrap_or_default()}
                    </Suspense>
                </span>
                <span class="text-sm text-gray-400">
                    {move || t!(i18n, football_hits)}{football.hits}
                </span>
            </div>
        </div>
    }
}
