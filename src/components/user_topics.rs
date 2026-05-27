use crate::i18n::{t, use_i18n};
use crate::models::Topic;
use crate::shared::constant::{BADGE_BLUE_NO_UL, BADGE_GRAY_NO_UL, FLEX_WRAP_GAP};
use crate::shared::locale::LocaleA;
use leptos::either::Either;
use leptos::prelude::*;

/// 用户关键词/话题区块 — 管理页和公开页共用
#[component]
pub fn UserTopics(keywords: Vec<Topic>, topics: Vec<Topic>) -> impl IntoView {
    let i18n = use_i18n();
    if keywords.is_empty() && topics.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-6">
                {if !keywords.is_empty() {
                    Either::Left(view! {
                        <div class="mb-4">
                            <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, features_topics)}</p>
                            <div class=FLEX_WRAP_GAP>
                                {keywords.into_iter().map(|t| {
                                    let path = format!("/footballs?topic={}", crate::shared::common::record_key(&t.id));
                                    view! {
                                        <LocaleA href=path class=BADGE_BLUE_NO_UL>{t.name.clone()}</LocaleA>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                } else {
                    Either::Right(())
                }}
                {if !topics.is_empty() {
                    Either::Left(view! {
                        <div>
                            <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, related_topics)}</p>
                            <div class=FLEX_WRAP_GAP>
                                {topics.into_iter().map(|t| {
                                    let path = format!("/footballs?topic={}", crate::shared::common::record_key(&t.id));
                                    view! {
                                        <LocaleA href=path class=BADGE_GRAY_NO_UL>{t.name.clone()}</LocaleA>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                } else {
                    Either::Right(())
                }}
            </div>
        })
    }
}
