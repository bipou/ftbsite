use crate::i18n::t;
use crate::shared::common::Either3;
use crate::shared::constant::{SLIDE_SIZED_MD, SLIDE_SIZED_SM, TEXT_SUBTLE};
use crate::shared::locale::{LocaleA, is_valid_locale};
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{HashedStylesheet, MetaTags, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

use crate::components::{Footer, Nav, SlidePanel};
use crate::i18n::{I18nContextProvider, Locale, use_i18n};
use crate::pages::{
    admin::{
        AdminFootballDetailPage, AdminFootballsPage, AdminPage, AdminUserDetailPage, AdminUsersPage,
    },
    auth::{RegisterForm, SignInForm, UserActivatePage},
    footballs::FootballsPage,
    home::HomePage,
    users::UsersPage,
    write::WriteArticlePage,
};
use crate::shared::auth::{AuthResource, AuthSignal, get_auth_user};

/// 签入/注册面板模式
#[derive(Clone, Copy, PartialEq)]
pub enum AuthMode {
    SignIn,
    Register,
}

pub type AuthPanelSignal = RwSignal<Option<AuthMode>>;
use leptos_i18n::Locale as _;

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    #[cfg(feature = "oth")]
    let google_ad = view! {
        <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-2498669832870483" crossorigin="anonymous"></script>
    };
    #[cfg(not(feature = "oth"))]
    let google_ad = ();

    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
                <link rel="icon" href="/favicon.ico"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                <HashedStylesheet options/>
                <script>
                    {r#"(function(){var t=localStorage.getItem("theme");if(t==="light"){document.documentElement.classList.add("light")}else if(t==="dark"||(!t&&window.matchMedia("(prefers-color-scheme:dark)").matches)){document.documentElement.classList.add("dark")}})()"#}
                </script>
            </head>
            <body class="min-h-screen bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-100">
                <App/>
                {google_ad}
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Auth 资源：SSR 异步解析，不阻塞首屏渲染
    let auth_res: AuthResource = Resource::new(|| (), |_| get_auth_user());
    provide_context(auth_res);

    // Auth 信号：由 Effect 从 Resource 同步，Nav 读取信号不触发 Suspense
    let auth_signal: AuthSignal = RwSignal::new(None);
    provide_context(auth_signal);
    Effect::new(move |_| {
        auth_signal.set(auth_res.get().and_then(|r| r.ok()).flatten());
    });

    // 签入/注册面板控制信号
    let auth_panel: AuthPanelSignal = RwSignal::new(None);
    provide_context(auth_panel);

    view! {
        <I18nContextProvider>
            <Router>
                <SetLocaleFromUrl/>
                <Nav/>
                <Suspense fallback=|| view! { <LoadingFallback/> }>
                    <Routes fallback=|| view! { <NotFound/> }>
                        // 各页面组件不再包含 Nav/Footer，统一由此层提供
                        <Route path=path!("/")                            view=HomePage/>
                        <Route path=path!("/:locale/")                    view=HomePage/>
                        <Route path=path!("/:locale/footballs")           view=FootballsPage/>
                        <Route path=path!("/:locale/footballs/analysis/new") view=WriteArticlePage/>
                        <Route path=path!("/:locale/footballs/:id")       view=FootballsPage/>
                        <Route path=path!("/:locale/footballs/category/:cid") view=FootballsPage/>
                        <Route path=path!("/:locale/footballs/topic/:tid") view=FootballsPage/>
                        <Route path=path!("/:locale/users")               view=UsersPage/>
                        <Route path=path!("/:locale/users/:id")     view=UsersPage/>
                        <Route path=path!("/:locale/users/:id/activate")  view=UserActivatePage/>
                        <Route path=path!("/:locale/admin")               view=AdminPage/>
                        <Route path=path!("/:locale/admin/footballs")     view=AdminFootballsPage/>
                        <Route path=path!("/:locale/admin/footballs/:id")  view=AdminFootballDetailPage/>
                        <Route path=path!("/:locale/admin/users")         view=AdminUsersPage/>
                        <Route path=path!("/:locale/admin/users/:id") view=AdminUserDetailPage/>
                    </Routes>
                </Suspense>
                <Footer/>

                // ── 签入/注册底部滑出面板 ────────────────────────────────────
                {
                    let auth_panel = use_context::<AuthPanelSignal>();
                    let open = Signal::derive(move || {
                        auth_panel.as_ref().map(|ap| ap.get().is_some()).unwrap_or(false)
                    });
                    let on_close = Callback::new(move |_| {
                        if let Some(ref ap) = auth_panel {
                            ap.set(None);
                        }
                    });
                    let size = Signal::derive(move || match auth_panel.as_ref().and_then(|ap| ap.get()) {
                        Some(AuthMode::Register) => SLIDE_SIZED_MD.to_string(),
                        _ => SLIDE_SIZED_SM.to_string(),
                    });
                    view! {
                        <SlidePanel open=open on_close=on_close panel_class=size>
                            {move || {
                                let mode = auth_panel.as_ref().and_then(|ap| ap.get());
                                match mode {
                                    Some(AuthMode::SignIn) => Either3::Left(view! { <SignInForm/> }),
                                    Some(AuthMode::Register) => Either3::Right(Either::Left(view! { <RegisterForm/> })),
                                    None => Either3::Right(Either::Right(())),
                                }
                            }}
                        </SlidePanel>
                    }
                }
            </Router>
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
        <Title text=move || ["404 – ", &site_title!(i18n)].join("")/>
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center space-y-4 p-8">
                <h1 class="text-7xl font-bold text-blue-600">"404"</h1>
                <p class={["text-xl", TEXT_SUBTLE].join(" ")}>{move || t!(i18n, page_error_404)}</p>
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
