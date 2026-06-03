// 底部滑出面板
use crate::class;
#[cfg(feature = "oth")]
use crate::components::footer::AdBanner;
use crate::i18n::{t, use_i18n};
use crate::shared::constant::{SLIDE_BODY, SLIDE_CLOSE, SLIDE_OPEN, SLIDE_OVERLAY, SLIDE_PANEL};
use leptos::prelude::*;

#[component]
pub fn SlidePanel(
    open: Signal<bool>,
    on_close: Callback<()>,
    #[prop(into, default = Signal::derive(|| String::new()))] panel_class: Signal<String>,
    #[prop(default = true)] show_footer: bool,
    children: Children,
) -> impl IntoView {
    let i18n = use_i18n();
    let content = children();
    let cb = on_close.clone();
    #[cfg(feature = "oth")]
    let google_ads = view! { <AdBanner/> };
    #[cfg(not(feature = "oth"))]
    let google_ads = ();

    view! {
        <div
            class=move || match open.get() {
                true => class!(SLIDE_OVERLAY, SLIDE_OPEN),
                false => class!(SLIDE_OVERLAY),
            }
        ></div>
        <div class=move || match open.get() {
            true => class!(SLIDE_PANEL, SLIDE_OPEN, &panel_class.get()),
            false => class!(SLIDE_PANEL, &panel_class.get()),
        }>
            <button class=SLIDE_CLOSE on:click=move |_| cb.run(())>"✕"</button>
            <div class=SLIDE_BODY>
                {content}
                {show_footer.then(|| view! {
                    {google_ads}
                    <div class="text-center mt-6">
                        <button class="slide-close-bottom" on:click=move |_| on_close.run(())>
                            {move || t!(i18n, close)}
                        </button>
                    </div>
                    <p class="text-xs text-red-400 dark:text-red-500 text-center mt-1">
                        {move || t!(i18n, site_warn)}
                    </p>
                })}
            </div>
        </div>
    }
}
