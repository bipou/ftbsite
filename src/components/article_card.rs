use crate::components::football_card::{CatBadge, status_badge, status_class};
use crate::i18n::{t, t_display, use_i18n};
use crate::models::Football;
use crate::shared::constant::{FLEX_BETWEEN, HOVER_SHADOW, TEXT_SUBTLE, TEXT_XS_MUTED};
use crate::shared::fns::get_username_by_id;
use leptos::either::Either;
use leptos::prelude::*;

/// 用户 / AI 分析文章卡片
#[component]
pub fn ArticleCard(football: Football, on_click: Callback<String>) -> impl IntoView {
    let i18n = use_i18n();
    let title = football.title();
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
    let fid = crate::shared::common::record_key(&football.id).to_string();

    let status = football.status;
    let card_class = ["card", "p-4", HOVER_SHADOW, status_class(status), "min-w-0"].join(" ");
    let badge = status_badge(status);
    let badge_or_label = match (badge.is_empty(), is_ai) {
        (false, _) => badge.to_string(),
        (true, true) => untrack(|| t_display!(i18n, analysis_ai).to_string()),
        (true, false) => String::new(),
    };

    // 卡片点击回调：因 on:click 需闭包非 Callback，故每处内联

    // 类别
    let category = football.category;
    let cat_kid = category
        .as_ref()
        .map(|c| crate::shared::common::record_key(&c.id).to_string());
    let cat_name = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category.as_ref().and_then(|c| c.name.get(&loc).cloned())
    });

    // 用户文章：获取作者 user_id
    let user_id = (football.ana_type == 0)
        .then(|| football.analyses.first())
        .flatten()
        .and_then(|a| a.user_id.clone());

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
                <button
                    class="font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 no-underline text-lg leading-tight min-w-0 border-0 bg-transparent cursor-pointer p-0 text-left"
                    on:click={
                        let fid = fid.clone();
                        let cb = on_click.clone();
                        move |_| cb.run(fid.clone())
                    }
                >
                    {title}
                </button>
                <span class="text-sm ml-2 whitespace-nowrap">
                    {badge_or_label}
                </span>
            </div>

            <div class=["text-sm", TEXT_SUBTLE, "mb-3", "space-x-2"].join(" ")>
                <CatBadge name=cat_name kid=cat_kid/>
                <span class="text-blue-500">{updated}</span>
            </div>

            {match summary {
                Some(s) => Either::Left(view! {
                    <button class="no-underline border-0 bg-transparent cursor-pointer p-0 text-left w-full" on:click={
                        let fid = fid.clone();
                        let cb = on_click.clone();
                        move |_| cb.run(fid.clone())
                    }>
                        <p class="text-sm text-gray-600 dark:text-gray-400 my-0">{s}</p>
                    </button>
                }),
                None => Either::Right(()),
            }}

            <div class=[FLEX_BETWEEN, "mt-3"].join(" ")>
                <span class=["text-sm", TEXT_XS_MUTED].join(" ")>
                    <Suspense fallback=|| "">
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
