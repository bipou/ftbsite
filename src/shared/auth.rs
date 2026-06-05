use crate::models::AuthUser;
use leptos::prelude::*;

/// JWT cookie auth check — runs server-side.
#[server]
pub async fn get_auth_user() -> Result<Option<AuthUser>, ServerFnError> {
    use crate::server::auth::{decode_jwt, get_cookie_value};
    use axum::http::HeaderMap;

    let headers: HeaderMap = leptos_axum::extract().await?;
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let token = match get_cookie_value(cookie, "fs_token") {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(None),
    };

    match decode_jwt(&token) {
        Ok(claims) => Ok(Some(AuthUser {
            username: claims.username,
            token,
        })),
        Err(_) => Ok(None),
    }
}

pub type AuthResource = Resource<Result<Option<AuthUser>, ServerFnError>>;
pub type AuthSignal = RwSignal<Option<AuthUser>>;

/// Call inside any reactive scope to get the current user (if signed in).
pub fn use_auth() -> Option<AuthUser> {
    use_context::<AuthSignal>().and_then(|s| s.get())
}
