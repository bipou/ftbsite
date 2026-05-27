use crate::i18n::{t, use_i18n};
use leptos::either::Either;
use leptos::prelude::*;

const PROSE_CLASS: &str = "prose prose-sm dark:prose-invert max-w-none";

/// 用户简介区块 — 管理页和公开页共用
#[component]
pub fn UserIntro(intro_html: String) -> impl IntoView {
    let i18n = use_i18n();
    if intro_html.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-6 mb-6">
                <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-3">
                    {move || t!(i18n, user_intro)}
                </h2>
                <article class=PROSE_CLASS inner_html=intro_html/>
            </div>
        })
    }
}
