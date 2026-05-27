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
                let name = cat.name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default();
                if let Some(ref sel) = selected {
                    let s = sel.clone();
                    let k = kid.clone();
                    let k2 = k.clone();
                    Either::Left(view! {
                        <button type="button"
                            class=move || if s.get() == k { BADGE_BLUE_NO_UL.to_string() } else { CAT_BTN.to_string() }
                            on:click=move |_| s.set(k2.clone())
                        >{name}</button>
                    })
                } else {
                    let url = format!("/{}/footballs?category={}", loc_str.get(), kid);
                    Either::Right(view! {
                        <a href=url class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700">{name}</a>
                    })
                }
            }).collect::<Vec<_>>()}
            {move || if expanded.get() {
                Either::Left(view! {
                    {high.clone().into_iter().map(|cat| {
                        let kid = record_key(&cat.id).to_string();
                        let name = cat.name.get(&i18n.get_locale().to_string()).cloned().unwrap_or_default();
                        if let Some(ref sel) = selected {
                            let s = sel.clone();
                            let k = kid.clone();
                            let k2 = k.clone();
                            Either::Left(view! {
                                <button type="button"
                                    class=move || if s.get() == k { BADGE_BLUE_NO_UL.to_string() } else { CAT_BTN.to_string() }
                                    on:click=move |_| s.set(k2.clone())
                                >{name}</button>
                            })
                        } else {
                            let url = format!("/{}/footballs?category={}", loc_str.get(), kid);
                            Either::Right(view! {
                                <a href=url class="text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700">{name}</a>
                            })
                        }
                    }).collect::<Vec<_>>()}
                })
            } else {
                Either::Right(())
            }}
            {if has_more {
                Either::Left(view! {
                    <button type="button"
                        class=CAT_BTN_MORE
                        on:click=move |_| set_expanded.update(|v| *v = !*v)
                    >
                        {move || if expanded.get() {
                            Either::Left(view! { {t_display!(i18n, collapse).to_string()} })
                        } else {
                            Either::Right(view! { {t_display!(i18n, expand).to_string()} })
                        }}
                    </button>
                })
            } else {
                Either::Right(())
            }}
        </>
    }
}
