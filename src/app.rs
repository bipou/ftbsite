use crate::i18n::t;
use crate::shared::constant::TEXT_SUBTLE;
use crate::shared::locale::{LocaleA, is_valid_locale};
use crate::site_title;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

use crate::i18n::{I18nContextProvider, Locale, use_i18n};
use crate::models::AuthUser;
use crate::pages::{
    admin::{AdminFootballDetailPage, AdminFootballsPage, AdminPage},
    auth::{RegisterPage, SignInPage, SignOutPage, UserActivatePage},
    football::FootballDetailPage,
    footballs::FootballsPage,
    home::HomePage,
    users::{UserProfilePage, UsersPage},
};
use leptos_i18n::Locale as _;

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
                <link rel="icon" href="/favicon.ico"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                <script>
                    {r#"(function(){var t=localStorage.getItem("theme");if(t==="light"){document.documentElement.classList.add("light")}else if(t==="dark"||(!t&&window.matchMedia("(prefers-color-scheme:dark)").matches)){document.documentElement.classList.add("dark")}})()"#}
                </script>
            </head>
            <body class="min-h-screen bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-100">
                <App/>
                <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-2498669832870483" crossorigin="anonymous"></script>
            </body>
        </html>
    }
}

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

/// Call inside any reactive scope to get the current user (if signed in).
pub fn use_auth() -> Option<AuthUser> {
    use_context::<AuthResource>()
        .and_then(|r| r.get())
        .and_then(|r| r.ok())
        .flatten()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Auth 资源：SSR 异步解析，不阻塞首屏渲染
    let auth_res: AuthResource = Resource::new(|| (), |_| get_auth_user());
    provide_context(auth_res);

    view! {
        <Stylesheet id="leptos" href="/pkg/football_site.css"/>

        <I18nContextProvider>
            <Suspense fallback=|| view! { <LoadingFallback/> }>
                <Router>
                    <SetLocaleFromUrl/>
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=path!("/")                            view=HomePage/>
                        <Route path=path!("/:locale/")                    view=HomePage/>
                        <Route path=path!("/:locale/register")            view=RegisterPage/>
                        <Route path=path!("/:locale/sign-in")             view=SignInPage/>
                        <Route path=path!("/:locale/sign-out")            view=SignOutPage/>
                        <Route path=path!("/:locale/rand")                view=FootballDetailPage/>
                        <Route path=path!("/:locale/footballs")           view=FootballsPage/>
                        <Route path=path!("/:locale/footballs/:id")       view=FootballDetailPage/>
                        <Route path=path!("/:locale/users")               view=UsersPage/>
                        <Route path=path!("/:locale/users/:username")     view=UserProfilePage/>
                        <Route path=path!("/:locale/users/:id/activate")  view=UserActivatePage/>
                        <Route path=path!("/:locale/admin")               view=AdminPage/>
                        <Route path=path!("/:locale/admin/footballs")     view=AdminFootballsPage/>
                        <Route path=path!("/:locale/admin/football/:id")  view=AdminFootballDetailPage/>
                    </Routes>
                </Router>
            </Suspense>
        </I18nContextProvider>
    }
}

/// 监听 URL 首个路径段，自动设置/切换语言
#[component]
fn SetLocaleFromUrl() -> impl IntoView {
    let i18n = use_i18n();
    let location = use_location();

    Effect::new(move |_| {
        let path = location.pathname.get();
        let first = path.trim_start_matches('/').split('/').next().unwrap_or("");
        if is_valid_locale(first) {
            if let Some(loc) = Locale::get_all().iter().find(|l| l.to_string() == first) {
                if i18n.get_locale() != *loc {
                    i18n.set_locale(*loc);
                }
            }
        }
    });
}

#[component]
fn NotFound() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <Title text=move || format!("404 – {}", site_title!(i18n))/>
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center space-y-4 p-8">
                <h1 class="text-7xl font-bold text-blue-600">"404"</h1>
                <p class={format!("text-xl {}", TEXT_SUBTLE)}>{move || t!(i18n, error_404)}</p>
                <LocaleA href="/" class="btn-primary inline-block mt-4">{move || t!(i18n, go_home)}</LocaleA>
            </div>
        </div>
    }
}

#[component]
fn LoadingFallback() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <p class="p-4 text-center">{move || t!(i18n, loading)}</p>
    }
}
