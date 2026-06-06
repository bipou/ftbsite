// 底部滑出面板
use crate::class;
#[cfg(feature = "oth")]
use crate::components::footer::AdBanner;
use crate::shared::constant::{SLIDE_BODY, SLIDE_CLOSE, SLIDE_OPEN, SLIDE_OVERLAY, SLIDE_PANEL};
use leptos::prelude::*;

#[component]
pub fn SlidePanel(
    open: Signal<bool>,
    on_close: Callback<()>,
    #[prop(into, default = Signal::derive(|| String::new()))] panel_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let content = children();
    let cb = on_close.clone();
    let ad_trigger = RwSignal::new(0u32);
    let _ = Effect::new(move |_| {
        if open.get() {
            ad_trigger.update(|v| *v += 1);
        }
    });
    #[cfg(feature = "oth")]
    let google_ads = view! { <AdBanner trigger=ad_trigger/> };
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
                {google_ads}
            </div>
        </div>
    }
}
