use crate::app::use_auth;
use crate::i18n::t;
use crate::i18n::{Locale, td_string, use_i18n};
use crate::shared::locale::{LocaleA, use_locale};
use leptos::either::Either;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen(inline_js = r#"
    export function toggle_theme() {
        const h = document.documentElement;
        if (h.classList.contains("dark")) {
            h.classList.remove("dark");
            h.classList.add("light");
            localStorage.setItem("theme", "light");
        } else {
            h.classList.remove("light");
            h.classList.add("dark");
            localStorage.setItem("theme", "dark");
        }
    }
"#)]
extern "C" {
    fn toggle_theme();
}

use crate::shared::constant::{BG_CARD, FLEX_BETWEEN, HOVER_NO_UNDERLINE, NO_UNDERLINE};
use leptos_i18n::Locale as LocaleTrait;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
fn Logo() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <span class="inline-flex items-center">
            <LocaleA href="/" class=["font-bold text-blue-600 dark:text-blue-400 text-2xl site-title", NO_UNDERLINE, HOVER_NO_UNDERLINE].join(" ")>
                {move || t!(i18n, site_name)}
            </LocaleA>
            <a href="/doc" class="inline-flex items-center justify-center text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 h-6 px-2 ml-2 no-underline" target="_blank" rel="noopener noreferrer">
                {move || t!(i18n, site_slogan)}
            </a>
        </span>
    }
}

#[component]
fn NavLinks() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <LocaleA href="/footballs">
            {move || t!(i18n, nav_football)}
        </LocaleA>
        <LocaleA href="/users">
            {move || t!(i18n, nav_user)}
        </LocaleA>
    }
}

#[component]
fn NavLeft() -> impl IntoView {
    view! {
        <Logo/>
        <div class="nav-links hidden sm:flex items-center gap-5 text-base">
            <NavLinks/>
        </div>
    }
}

#[component]
fn LangDropdown() -> impl IntoView {
    let i18n = use_i18n();
    let (open, set_open) = signal(false);
    let navigate = use_navigate();
    let nav = navigate.clone();
    let loc_str = use_locale();
    let location = use_location();
    view! {
        <div class="relative inline-block">
            <button
                class={["px-2 py-1 text-sm border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200", BG_CARD, "hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"].join(" ")}
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
            >
                "🌐 "
                <span class="hidden sm:inline">{move || t!(i18n, lang)}</span>
                <span class="hidden sm:inline ml-1 opacity-50">"▾"</span>
            </button>
            <div
                class=move || ["lang-list border border-gray-200 dark:border-gray-700 rounded shadow-md py-1", BG_CARD, if open.get() { "" } else { "hidden" }, "absolute top-full left-1/2 -translate-x-1/2 mt-1 whitespace-nowrap z-50"].join(" ")
            >
                {Locale::get_all().iter().map(|&locale| {
                    let new_loc = locale.to_string();
                    view! {
                        <button
                            on:click={
                                let navigate = nav.clone();
                                let new_loc = new_loc.clone();
                                move |_| {
                                    let old = loc_str.get_untracked();
                                    let path = location.pathname.get_untracked();
                                    let rest = path.strip_prefix(&["/", &old].join("")).unwrap_or(&path).to_string();
                                    i18n.set_locale(locale);
                                    navigate(&["/", &new_loc, &rest].join(""), Default::default());
                                    set_open.set(false);
                                }
                            }
                            class=move || if i18n.get_locale() == locale { "lang-active" } else { "" }
                        >
                            {td_string!(locale, lang)}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let on_click = move |_| {
        #[cfg(feature = "hydrate")]
        {
            toggle_theme();
        }
    };
    view! {
        <button
            on:click=on_click
            class="w-6 h-6 flex items-center justify-center rounded-full border-0 bg-transparent cursor-pointer"
        >
            "☀️"
        </button>
    }
}

#[component]
fn AuthSection() -> impl IntoView {
    let i18n = use_i18n();
    move || {
        let auth = use_auth();
        if let Some(user) = auth {
            Either::Left(view! {
                <span class="text-gray-700 dark:text-gray-200 font-medium hidden sm:inline text-base">
                    {user.username}
                </span>
                <LocaleA href="/sign-out" class=["text-sm text-gray-500 hover:text-red-500", NO_UNDERLINE].join(" ")>
                    {move || t!(i18n, sign_out)}
                </LocaleA>
            })
        } else {
            Either::Right(view! {
                <LocaleA href="/sign-in" class="nav-links text-sm">
                    {move || t!(i18n, sign_in)}
                </LocaleA>
                <LocaleA href="/register" class="nav-links text-sm">
                    {move || t!(i18n, register)}
                </LocaleA>
            })
        }
    }
}

#[component]
fn HamburgerMenu() -> impl IntoView {
    let (open, set_open) = signal(false);
    let i18n = use_i18n();
    let close = Callback::new(move |_: leptos::ev::MouseEvent| set_open.set(false));

    view! {
        <div class="sm:hidden">
            <button
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
                class="w-8 h-8 flex items-center justify-center border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200 bg-transparent cursor-pointer text-lg shrink-0"
            >
                {move || if open.get() { "✕" } else { "☰" }}
            </button>

            {move || if open.get() {
                Some(view! { <div class="fixed inset-0 z-40" on:click=move |_| set_open.set(false)></div> })
            } else {
                None
            }}

            <div
                class=move || ["hm-menu border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg absolute top-full mt-2 z-50 whitespace-nowrap", BG_CARD, if open.get() { "" } else { "hidden" }].join(" ")
                style="right:1rem"
            >
                <div class="px-4 py-3 flex flex-col gap-2">
                    <div class="nav-links flex flex-col gap-2">
                        <LocaleA href="/footballs" on_click=close>
                            {move || t!(i18n, nav_football)}
                        </LocaleA>
                        <LocaleA href="/users" on_click=close>
                            {move || t!(i18n, nav_user)}
                        </LocaleA>
                    </div>
                </div>
                <hr/>
                <div class="px-4 py-3 flex flex-col gap-2">
                    {move || {
                        let auth = use_auth();
                        if let Some(user) = auth {
                            Either::Left(view! {
                                <span class="text-gray-700 dark:text-gray-200 font-medium text-base">
                                    {user.username}
                                </span>
                                <LocaleA href="/sign-out" on_click=close class="hm-signout text-sm hover:text-red-500">
                                    {move || t!(i18n, sign_out)}
                                </LocaleA>
                            })
                        } else {
                            Either::Right(view! { <div class="nav-links text-sm flex flex-col gap-2">
                                <LocaleA href="/sign-in" on_click=close>
                                    {move || t!(i18n, sign_in)}
                                </LocaleA>
                                <LocaleA href="/register" on_click=close>
                                    {move || t!(i18n, register)}
                                </LocaleA>
                            </div> })
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn NavRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 text-sm">
            <ThemeToggle/>
            <LangDropdown/>
            <div class="hidden sm:flex items-center gap-3">
                <AuthSection/>
            </div>
            <HamburgerMenu/>
        </div>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class={[BG_CARD, "border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 shadow-sm"].join(" ")}>
            <div class="max-w-6xl mx-auto px-4">
                <div class={[FLEX_BETWEEN, "h-12"].join(" ")}>
                    <NavLeft/>
                    <NavRight/>
                </div>
            </div>
        </nav>
    }
}
