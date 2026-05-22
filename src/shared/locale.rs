// 语言工具模块：统一管理 URL 前缀、链接生成，全程不硬编码语言代码
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use crate::i18n::Locale;
use leptos_i18n::Locale as _;

/// 从 URL 的 :locale 段获取当前语言字符串
/// 仅在 <Router> 内部可用；无语言则以 default_locale() 回退
pub fn use_locale_str() -> Memo<String> {
    let params = use_params_map();
    Memo::new(move |_| {
        params
            .read()
            .get("locale")
            .unwrap_or_else(|| default_locale().to_string())
    })
}

/// 默认语言代码 = Locale::get_all() 的第一项（由 build.rs 决定顺序）
pub fn default_locale() -> Locale {
    Locale::get_all()
        .first()
        .copied()
        .expect("至少注册一种语言")
}

/// 语言代码列表
pub fn locale_codes() -> Vec<String> {
    Locale::get_all().iter().map(|l| l.to_string()).collect()
}

/// 校验给定字符串是否为支持的语言代码
pub fn is_valid_locale(code: &str) -> bool {
    Locale::get_all().iter().any(|l| l.to_string() == code)
}

/// 非组件上下文：拼接带语言前缀的完整路径
pub fn locale_href(locale_str: &str, path: &str) -> String {
    format!("/{}{}", locale_str, path)
}

/// 语言感知链接组件 — 替代 leptos_router::A
/// 用法：<LocaleA href="/footballs">文本</LocaleA>
/// 自动在 href 前加上当前语言前缀
#[component]
pub fn LocaleA(
    #[prop(into)] href: String,
    #[prop(into, optional)] class: Option<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] target: Option<&'static str>,
    #[prop(optional)] rel: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let loc = use_locale_str();
    let full = move || format!("/{}{}", loc.get(), href);
    view! {
        <A
            href=full
            attr:class=class.unwrap_or_default()
            attr:target=target.unwrap_or_default()
            attr:rel=rel.unwrap_or_default()
            on:click=move |ev| {
                if let Some(ref cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </A>
    }
}
