use leptos::prelude::*;

/// 编辑器预览端点
#[server]
pub async fn preview_md(md: String) -> Result<String, ServerFnError> {
    use crate::server::markdown::render_md;
    Ok(render_md(&md))
}

/// 获取上传 nonce（有效期 30 分钟）
#[server]
pub async fn get_upload_nonce() -> Result<String, ServerFnError> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use crate::server::upload::{NONCE_COUNTER, NONCE_STORE};

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

/// 上传图片到 tmp 目录。须持有有效 nonce
#[server]
pub async fn upload_image(
    data_url: String,
    _filename: String,
    nonce: String,
    scope: String,
) -> Result<String, ServerFnError> {
    use crate::server::upload::{save_upload, verify_nonce};
    verify_nonce(&nonce)?;
    save_upload(&data_url, &scope)
}
