use crate::i18n::{t_display, use_i18n};
use crate::models::Category;
use crate::shared::common::record_key;
use crate::shared::constant::{BADGE_BLUE_NO_UL, CAT_BTN, CAT_BTN_MORE};
use crate::shared::locale::use_locale;
use leptos::either::Either;
use leptos::prelude::*;

/// 通用分类选择器
/// - filter 模式：`selected` 为 None，渲染 `<a>` 链接跳转
/// - select 模式：`selected` 为 Some(RwSignal)，渲染 `<button>` 单选
#[component]
pub fn CategorySelect(
    all: Vec<Category>,
    #[prop(into, default = None)] selected: Option<RwSignal<String>>,
    #[prop(default = false)] expandable: bool,
) -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let (expanded, set_expanded) = signal(false);

    let low: Vec<_> = all.iter().filter(|c| c.level <= 2).cloned().collect();
    let high: Vec<_> = all.iter().filter(|c| c.level > 2).cloned().collect();
    let has_more = expandable && !high.is_empty();

    view! {
        <>
            {low.into_iter().map(|cat| {
                let kid = record_key(&cat.id).to_string();
                let cat_name = cat.name.clone();
                match &selected {
                    Some(sel) => {
                        let s = *sel;
                        let k = kid.clone();
                        let k2 = kid.clone();
                        Either::Left(view! {
                            <button type="button"
                                class=move || if s.get() == k { BADGE_BLUE_NO_UL.to_string() } else { CAT_BTN.to_string() }
                                on:click=move |_| s.set(k2.clone())
                            >{move || cat_name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default()}</button>
                        })
                    }
                    None => Either::Right(view! {
                        <a href=move || ["/", &loc_str.get(), "/footballs/category/", &kid].join("") class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700">{move || cat_name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default()}</a>
                    }),
                }
            }).collect::<Vec<_>>()}
            {move || match expanded.get() {
                true => Either::Left(view! {
                    {high.clone().into_iter().map(|cat| {
                        let kid = record_key(&cat.id).to_string();
                        let cat_name = cat.name.clone();
                        match &selected {
                            Some(sel) => {
                                let s = *sel;
                                let k = kid.clone();
                                let k2 = kid.clone();
                                Either::Left(view! {
                                    <button type="button"
                                        class=move || if s.get() == k { BADGE_BLUE_NO_UL.to_string() } else { CAT_BTN.to_string() }
                                        on:click=move |_| s.set(k2.clone())
                                    >{move || cat_name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default()}</button>
                                })
                            }
                            None => Either::Right(view! {
                                <a href=move || ["/", &loc_str.get(), "/footballs/category/", &kid].join("") class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700">{move || cat_name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default()}</a>
                            }),
                        }
                    }).collect::<Vec<_>>()}
                }),
                false => Either::Right(()),
            }}
            {match has_more {
                true => Either::Left(view! {
                    <button type="button"
                        class=CAT_BTN_MORE
                        on:click=move |_| set_expanded.update(|v| *v = !*v)
                    >
                        {move || match expanded.get() {
                            true => Either::Left(view! { {t_display!(i18n, collapse).to_string()} }),
                            false => Either::Right(view! { {t_display!(i18n, more).to_string()} }),
                        }}
                    </button>
                }),
                false => Either::Right(()),
            }}
        </>
    }
}
