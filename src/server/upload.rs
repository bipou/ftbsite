use leptos::prelude::*;

// ── nonce 存储 ───────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub static NONCE_STORE: std::sync::OnceLock<Mutex<HashMap<String, Instant>>> =
    std::sync::OnceLock::new();

pub static NONCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const UPLOAD_TMP: &str = "/uploads/{scope}/tmp/imgs/{fname}";
const UPLOAD_DRAFT: &str = "/uploads/{scope}/draft/imgs/{fname}";
const UPLOAD_ACTIVE: &str = "/uploads/{scope}/active/imgs/{fname}";

/// 校验 nonce 是否存在且未过期（30 分钟）
pub fn verify_nonce(nonce: &str) -> Result<(), ServerFnError> {
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

// ── 图片上传 ─────────────────────────────────────────────────────────────

/// 解析 data URL 并写入 scope/tmp/imgs/，返回访问路径
pub fn save_upload(data_url: &str, scope: &str) -> Result<String, ServerFnError> {
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

    let dir = uploads_dir(scope, "tmp");
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

    Ok(UPLOAD_TMP
        .replace("{scope}", scope)
        .replace("{fname}", &fname))
}

/// 将 markdown 中 scope/tmp/ 图片 move 到 draft 或 active
pub fn move_uploads(md: &str, scope: &str, draft: bool) -> Result<String, ServerFnError> {
    let stage = if draft { "draft" } else { "active" };
    let dest_dir = uploads_dir(scope, stage);
    std::fs::create_dir_all(&dest_dir).map_err(|_| ServerFnError::new("upload_failed"))?;

    let tmp_dir = uploads_dir(scope, "tmp");
    let from = UPLOAD_TMP.replace("{scope}", scope).replace("/{fname}", "");
    let to = if draft {
        UPLOAD_DRAFT
            .replace("{scope}", scope)
            .replace("/{fname}", "")
    } else {
        UPLOAD_ACTIVE
            .replace("{scope}", scope)
            .replace("/{fname}", "")
    };

    let mut result = md.to_string();
    let mut search_from = 0;

    while let Some(pos) = result[search_from..].find(&from) {
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

    result = result.replace(&from, &to);
    Ok(result)
}

fn uploads_dir(scope: &str, stage: &str) -> std::path::PathBuf {
    std::env::var("LEPTOS_SITE_ROOT")
        .map(|root| {
            std::path::PathBuf::from(root)
                .join("uploads")
                .join(scope)
                .join(stage)
                .join("imgs")
        })
        .unwrap_or_else(|_| {
            std::path::PathBuf::from("public/uploads")
                .join(scope)
                .join(stage)
                .join("imgs")
        })
}
