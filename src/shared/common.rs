use crate::models::PageInfo;
use leptos::either::Either;

// ── Either type aliases ──────────────────────────────────────────────────

pub type Either3<A, B, C> = Either<A, Either<B, C>>;

/// 服务端错误码 → i18n 文本。未匹配则原样返回
#[macro_export]
macro_rules! server_error_text {
    ($i18n:expr, $raw:expr, $($code:literal => $key:ident),+ $(,)?) => {{
        let raw: &str = &$raw;
        let mut msg = raw.to_string();
        $(
            if raw.contains($code) { msg = $crate::i18n::t_display!($i18n, $key).to_string(); }
        )*
        msg
    }};
}

/// CSS 类列表拼接，自动过滤空串
#[macro_export]
macro_rules! class {
    ($($part:expr),+ $(,)?) => {
        [$($part),+].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
    };
}

/// 详情面板信号+回调 — 消除 3 页面重复
/// 带导航：use_detail_panel!(sel, fetch_fn, "/path", "/prefix/")
/// 无导航：use_detail_panel!(sel, fetch_fn)
#[macro_export]
macro_rules! use_detail_panel {
    ($sel:ident, $fetch_fn:path, $close_base:expr, $click_prefix:expr) => {
        let $sel: leptos::prelude::RwSignal<Option<String>> = leptos::prelude::RwSignal::new(None);
        let detail_open = leptos::prelude::Signal::derive(move || $sel.get().is_some());
        let detail_data = leptos::prelude::Resource::new(
            move || $sel.get(),
            |id| async move {
                match id.filter(|s| !s.is_empty()) {
                    Some(id) => $fetch_fn(id).await,
                    _ => Ok(None),
                }
            },
        );
        let detail_close = {
            let navigate = navigate.clone();
            let loc_str = loc_str.clone();
            leptos::prelude::Callback::new(move |_| {
                $sel.set(None);
                navigate(
                    &["/", &loc_str.get(), $close_base].join(""),
                    Default::default(),
                );
            })
        };
        let on_card_click = {
            let navigate = navigate.clone();
            let loc_str = loc_str.clone();
            leptos::prelude::Callback::new(move |id: String| {
                $sel.set(Some(id.clone()));
                navigate(
                    &["/", &loc_str.get(), $click_prefix, &id].join(""),
                    Default::default(),
                );
            })
        };
    };
    ($sel:ident, $fetch_fn:path) => {
        let $sel: leptos::prelude::RwSignal<Option<String>> = leptos::prelude::RwSignal::new(None);
        let detail_open = leptos::prelude::Signal::derive(move || $sel.get().is_some());
        let detail_data = leptos::prelude::Resource::new(
            move || $sel.get(),
            |id| async move {
                match id.filter(|s| !s.is_empty()) {
                    Some(id) => $fetch_fn(id).await,
                    _ => Ok(None),
                }
            },
        );
        let detail_close = leptos::prelude::Callback::new(move |_| {
            $sel.set(None);
        });
        let on_card_click = leptos::prelude::Callback::new(move |id: String| {
            $sel.set(Some(id));
        });
    };
}

/// 详情关闭回调 — 导航回列表 + 清 ID
#[macro_export]
macro_rules! detail_close_nav {
    ($sel:expr) => {{
        let sel = $sel;
        leptos::prelude::Callback::new(move |_| {
            sel.set(None);
        })
    }};
    ($sel:expr, $i18n:expr, $base:expr) => {{
        let sel = $sel;
        leptos::prelude::Callback::new(move |_| {
            #[cfg(feature = "hydrate")]
            {
                let url = ["/", &$i18n.get_locale().to_string(), $base].join("");
                let _ = web_sys::window()
                    .unwrap()
                    .history()
                    .unwrap()
                    .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
            sel.set(None);
        })
    }};
}

/// 详情打开回调 — replaceState 写 URL + 设信号 + 递增版本
#[macro_export]
macro_rules! detail_open_nav {
    ($sel:expr, $ver:expr, $i18n:expr, $prefix:expr) => {{
        let sel = $sel;
        let ver = $ver;
        leptos::prelude::Callback::new(move |fid: String| {
            #[cfg(feature = "hydrate")]
            {
                let url = ["/", &$i18n.get_locale().to_string(), $prefix, &fid].join("");
                let _ = web_sys::window()
                    .unwrap()
                    .history()
                    .unwrap()
                    .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
            sel.set(Some(fid));
            ver.update(|v| *v += 1);
        })
    }};
}

// ── Page title macros ────────────────────────────────────────────────────

#[macro_export]
macro_rules! page_title {
    ($i18n:expr, $key:ident) => {
        $crate::i18n::t_display!(
            $i18n,
            page_title,
            key = $crate::i18n::t_display!($i18n, $key),
            name = $crate::i18n::t_display!($i18n, site_name),
            slogan = $crate::i18n::t_display!($i18n, site_slogan)
        )
        .to_string()
    };
}

#[macro_export]
macro_rules! site_title {
    ($i18n:expr) => {
        $crate::i18n::t_display!(
            $i18n,
            site_title,
            name = $crate::i18n::t_display!($i18n, site_name),
            slogan = $crate::i18n::t_display!($i18n, site_slogan)
        )
        .to_string()
    };
}

// ── Pagination（client + server）──────────────────────────────────────────

pub fn make_page_info(from: i64, ps: i64, total: u64) -> PageInfo {
    let tp = ((total as f64 / ps as f64).ceil() as u32).max(1);
    PageInfo {
        current_page: from as u32,
        total_pages: tp,
        total_count: total,
        has_previous: from > 1,
        has_next: (from as u32) < tp,
    }
}

// ── RecordId helpers（client + server）────────────────────────────────────

/// 从 "table:id" 提取裸 key，用于 URL 路径/查询参数
pub fn record_key(full: &str) -> &str {
    full.rsplit_once(':').map(|(_, k)| k).unwrap_or(full)
}

// ── SurrealDB helpers（server only）───────────────────────────────────────

#[cfg(feature = "ssr")]
use surrealdb::types::{RecordId, RecordIdKey};

#[cfg(feature = "ssr")]
/// RecordIdKey → 纯字符串
fn key_str(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => format!("{key:?}"),
    }
}

#[cfg(feature = "ssr")]
/// RecordId → "table:id" 字符串
pub fn rid_str(r: &RecordId) -> String {
    format!("{}:{}", r.table, key_str(&r.key))
}

#[cfg(feature = "ssr")]
/// 字符串 → RecordId。含 `:` 则按 "table:key" 解析，否则用 default_table 前缀
pub fn into_rid(input: &str, default_table: &str) -> RecordId {
    if let Some((tbl, key)) = input.split_once(':') {
        RecordId::new(tbl, key)
    } else {
        RecordId::new(default_table, input)
    }
}

// ── Datetime helpers（server only）────────────────────────────────────────

#[cfg(feature = "ssr")]
/// 格式化为 "%Y-%m-%d"（UTC+8）
pub fn ymd8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%Y-%m-%d").to_string()
}

#[cfg(feature = "ssr")]
/// 格式化为 "%Y-%m-%d %H:%M:%S%:z"（UTC+8）
pub fn ymdhmsz8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8)
        .format("%Y-%m-%d %H:%M:%S%:z")
        .to_string()
}
