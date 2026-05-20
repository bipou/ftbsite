use crate::models::PageInfo;
use leptos::either::Either;
use leptos::prelude::*;

// ── Server functions (client ↔ server) ─────────────────────────────────────

/// 编辑器预览端点
#[server]
pub async fn preview_md(md: String) -> Result<String, ServerFnError> {
    use crate::server::markdown::render_md;
    Ok(render_md(&md))
}

// ── nonce 存储（server only）────────────────────────────────────────────

#[cfg(feature = "ssr")]
use std::{collections::HashMap, sync::Mutex, time::Instant};

#[cfg(feature = "ssr")]
static NONCE_STORE: std::sync::OnceLock<Mutex<HashMap<String, Instant>>> =
    std::sync::OnceLock::new();
#[cfg(feature = "ssr")]
static NONCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "ssr")]
fn verify_nonce(nonce: &str) -> Result<(), ServerFnError> {
    use std::time::Duration;
    let map = NONCE_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut m = map.lock().unwrap();
    m.retain(|_, t| t.elapsed() < Duration::from_secs(1800));
    if m.contains_key(nonce) {
        Ok(())
    } else {
        Err(ServerFnError::new("upload_failed"))
    }
}

/// 获取上传 nonce，有效期 30 分钟
#[server]
pub async fn get_upload_nonce() -> Result<String, ServerFnError> {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let map = NONCE_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut m = map.lock().unwrap();
    m.retain(|_, t| t.elapsed() < Duration::from_secs(1800));

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cnt = NONCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nonce = format!("{ts:x}{cnt:x}");
    m.insert(nonce.clone(), Instant::now());
    Ok(nonce)
}

/// 上传图片到 tmp。须持有有效 nonce。
#[server]
pub async fn upload_image(
    data_url: String,
    _filename: String,
    nonce: String,
) -> Result<String, ServerFnError> {
    verify_nonce(&nonce)?;
    save_upload(&data_url)
}

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
            $crate::i18n::t_string!($i18n, $key),
            $crate::i18n::t_string!($i18n, site_name),
            $crate::i18n::t_string!($i18n, site_slogan)
        )
    };
}

#[macro_export]
macro_rules! site_title {
    ($i18n:expr) => {
        format!(
            "{} | {}",
            $crate::i18n::t_string!($i18n, site_name),
            $crate::i18n::t_string!($i18n, site_slogan)
        )
    };
}

// ── Pagination (wasm + server) ────────────────────────────────────────────────

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

// ── RecordId helpers (client + server) ────────────────────────────────────────

/// Extract the bare key from a "table:id" string, for URL path / query generation.
/// `"footballs:abc123"` → `"abc123"`
/// `"abc123"`           → `"abc123"` (passthrough for bare keys)
pub fn record_key(full: &str) -> &str {
    full.rsplit_once(':').map(|(_, k)| k).unwrap_or(full)
}

// ── SurrealDB helpers (server only) ───────────────────────────────────────────

#[cfg(feature = "ssr")]
use surrealdb::types::{RecordId, RecordIdKey};

#[cfg(feature = "ssr")]
/// `RecordIdKey` → plain string (internal helper)
fn key_str(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => format!("{key:?}"),
    }
}

#[cfg(feature = "ssr")]
/// `RecordId` → `"table:id"` string (use this instead of Display, which RecordId lacks)
pub fn rid_str(r: &RecordId) -> String {
    format!("{}:{}", r.table, key_str(&r.key))
}

#[cfg(feature = "ssr")]
/// Parse a string into `RecordId`.
/// If `input` contains `:` it is parsed as `"table:key"`; otherwise `default_table` is prepended.
pub fn into_rid(input: &str, default_table: &str) -> RecordId {
    if let Some((tbl, key)) = input.split_once(':') {
        RecordId::new(tbl, key)
    } else {
        RecordId::new(default_table, input)
    }
}

// ── Datetime helpers (server only) ────────────────────────────────────────────

/// Format a SurrealDB Datetime as "%Y-%m-%d" in UTC+8.
#[cfg(feature = "ssr")]
pub fn ymd8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%Y-%m-%d").to_string()
}

/// Format a SurrealDB Datetime as "%Y-%m-%d %H:%M:%S%:z" in UTC+8.
#[cfg(feature = "ssr")]
pub fn ymdhmsz8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8)
        .format("%Y-%m-%d %H:%M:%S%:z")
        .to_string()
}

// ── 图片上传（server only）────────────────────────────────────────────────

#[cfg(feature = "ssr")]
/// 解析 data URL 并写入 tmp/images/，返回访问路径
pub fn save_upload(data_url: &str) -> Result<String, ServerFnError> {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use std::io::Write;

    let (mime, b64) = data_url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or_else(|| ServerFnError::new("upload_failed"))?;

    let ext = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => return Err(ServerFnError::new("upload_failed")),
    };

    let bytes = STANDARD
        .decode(b64.as_bytes())
        .map_err(|_| ServerFnError::new("upload_failed"))?;

    if bytes.len() > 5 * 1024 * 1024 {
        return Err(ServerFnError::new("upload_failed"));
    }

    let dir = uploads_tmp_dir();
    std::fs::create_dir_all(&dir).map_err(|_| ServerFnError::new("upload_failed"))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let fname = format!("{ts}.{ext}");

    let path = dir.join(&fname);
    let mut f = std::fs::File::create(&path).map_err(|_| ServerFnError::new("upload_failed"))?;
    f.write_all(&bytes)
        .map_err(|_| ServerFnError::new("upload_failed"))?;

    Ok(format!("/uploads/tmp/images/{fname}"))
}

#[cfg(feature = "ssr")]
/// 将 markdown 中 /uploads/tmp/images/xxx → rename 到 uploads/active/images/，替换路径
pub fn move_uploads(md: &str) -> Result<String, ServerFnError> {
    let dest_dir = uploads_active_dir();
    std::fs::create_dir_all(&dest_dir).map_err(|_| ServerFnError::new("upload_failed"))?;

    let tmp_dir = uploads_tmp_dir();
    let from = "/uploads/tmp/images/";
    let to = "/uploads/active/images/";

    let mut result = md.to_string();
    let mut search_from = 0;

    while let Some(pos) = result[search_from..].find(from) {
        let abs = search_from + pos;
        let rest = &result[abs + from.len()..];
        let fname_len = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_')
            .unwrap_or(rest.len());
        if fname_len > 0 {
            let fname = &rest[..fname_len];
            let src = tmp_dir.join(fname);
            let dst = dest_dir.join(fname);
            if src.exists() {
                std::fs::rename(&src, &dst).map_err(|_| ServerFnError::new("upload_failed"))?;
            }
        }
        search_from = abs + from.len() + fname_len;
    }

    result = result.replace(from, to);
    Ok(result)
}

#[cfg(feature = "ssr")]
fn uploads_tmp_dir() -> std::path::PathBuf {
    std::env::var("LEPTOS_SITE_ROOT")
        .map(|root| {
            std::path::PathBuf::from(root)
                .join("uploads")
                .join("tmp")
                .join("images")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("public/uploads/tmp/images"))
}

#[cfg(feature = "ssr")]
fn uploads_active_dir() -> std::path::PathBuf {
    std::env::var("LEPTOS_SITE_ROOT")
        .map(|root| {
            std::path::PathBuf::from(root)
                .join("uploads")
                .join("active")
                .join("images")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("public/uploads/active/images"))
}
