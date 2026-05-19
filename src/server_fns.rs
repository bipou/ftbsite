use leptos::prelude::*;

/// 编辑器预览端点 —— 客户端调服务端
#[server]
pub async fn preview_md(md: String) -> Result<String, ServerFnError> {
    use crate::server::markdown::render_md;
    Ok(render_md(&md))
}

/// 上传图片。接收客户端发的 data URL，校验后写入文件，返回访问路径。
#[server]
pub async fn upload_image(data_url: String, filename: String) -> Result<String, ServerFnError> {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use std::io::Write;

    // 解析 data URL: "data:image/png;base64,iVBOR..."
    let (mime, b64) = data_url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or_else(|| ServerFnError::new("非法的 data URL"))?;

    let ext = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => return Err(ServerFnError::new("不支持的图片格式")),
    };

    let bytes = STANDARD
        .decode(b64.as_bytes())
        .map_err(|e| ServerFnError::new(format!("base64 解码失败: {e}")))?;

    if bytes.len() > 5 * 1024 * 1024 {
        return Err(ServerFnError::new("图片超过 5MB 上限"));
    }

    // 上传目录：优先 LEPTOS_SITE_ROOT（生产环境），回退 public
    let dir = std::env::var("LEPTOS_SITE_ROOT")
        .map(|root| std::path::PathBuf::from(root).join("uploads"))
        .unwrap_or_else(|_| std::path::PathBuf::from("public/uploads"));

    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("创建上传目录失败: {e}")))?;

    // 生成唯一文件名
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let safe_name: String = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let fname = format!("{ts}_{safe_name}.{ext}");

    let path = dir.join(&fname);
    let mut f = std::fs::File::create(&path)
        .map_err(|e| ServerFnError::new(format!("创建文件失败: {e}")))?;
    f.write_all(&bytes)
        .map_err(|e| ServerFnError::new(format!("写入文件失败: {e}")))?;

    Ok(format!("/uploads/{fname}"))
}
