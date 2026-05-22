use crate::models::PageInfo;
use leptos::either::Either;

// ── Either type aliases ──────────────────────────────────────────────────

pub type Either3<A, B, C> = Either<A, Either<B, C>>;
pub type Either5<A, B, C, D, E> = Either<A, Either<B, Either<C, Either<D, E>>>>;
pub type Either6<A, B, C, D, E, F> = Either<A, Either<B, Either<C, Either<D, Either<E, F>>>>>;

// ── Page title macros ────────────────────────────────────────────────────

#[macro_export]
macro_rules! page_title {
    ($i18n:expr, $key:ident) => {
        format!(
            "{} – {} | {}",
            $crate::i18n::t_display!($i18n, $key),
            $crate::i18n::t_display!($i18n, site_name),
            $crate::i18n::t_display!($i18n, site_slogan)
        )
    };
}

#[macro_export]
macro_rules! site_title {
    ($i18n:expr) => {
        format!(
            "{} | {}",
            $crate::i18n::t_display!($i18n, site_name),
            $crate::i18n::t_display!($i18n, site_slogan)
        )
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
