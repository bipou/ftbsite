use crate::i18n::{t, t_display, use_i18n};
use crate::pages::auth::get_captcha;
use leptos::html::Input;
use leptos::prelude::*;

// ── CaptchaState ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct CaptchaState {
    pub token: RwSignal<String>,
    pub answer: RwSignal<String>,
    pub ok: RwSignal<bool>,
    pub refresh_trigger: RwSignal<u32>,
}

impl CaptchaState {
    pub fn new() -> Self {
        Self {
            token: RwSignal::new(String::new()),
            answer: RwSignal::new(String::new()),
            ok: RwSignal::new(false),
            refresh_trigger: RwSignal::new(0),
        }
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

fn check_answer(input: &str, answer: u8) -> bool {
    input.trim().parse::<u8>().ok() == Some(answer)
}

// ── CaptchaCore 组件 ───────────────────────────────────────────────────────────
/// 需父组件先 provide_context(CaptchaState::new())，本组件读写其信号
#[component]
pub fn CaptchaCore() -> impl IntoView {
    let i18n = use_i18n();
    let state = use_context::<CaptchaState>().expect("CaptchaState not provided");
    let (status_msg, set_status_msg) = signal(String::new());
    let answer_ref = NodeRef::<Input>::new();

    let captcha_key = RwSignal::new(0);
    let captcha_res = Resource::new(
        move || captcha_key.get(),
        |_| async move { get_captcha().await.ok() },
    );
    captcha_key.set(1); // 组件挂载时立即触发请求

    let svg = move || {
        captcha_res
            .get()
            .flatten()
            .map(|(s, _, _)| s)
            .unwrap_or_default()
    };
    let answer = move || captcha_res.get().flatten().map(|(_, _, a)| a).unwrap_or(0);

    let on_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        state.answer.set(val.clone());
        let ans = answer();
        if val.is_empty() {
            set_status_msg.set(String::new());
            state.ok.set(false);
        } else if check_answer(&val, ans) {
            set_status_msg.set("✓".into());
            state.ok.set(true);
        } else {
            set_status_msg.set("✗".into());
            state.ok.set(false);
        }
    };

    // captcha 刷新时清除答案
    Effect::new(move |_| {
        let data = captcha_res.get();
        if let Some((_, t, _)) = data.flatten() {
            state.token.set(t);
        }
        state.answer.set(String::new());
        if let Some(input) = answer_ref.get() {
            let _ = input.set_value("");
        }
        state.ok.set(false);
        set_status_msg.set(String::new());
    });

    // 每5分钟自动刷新验证码
    set_interval(
        move || {
            captcha_res.refetch();
        },
        std::time::Duration::from_secs(300),
    );

    // 外部触发刷新（父组件递增 refresh_trigger）
    Effect::new(move |prev: Option<u32>| {
        if prev.is_some() {
            captcha_res.refetch();
        }
        state.refresh_trigger.get()
    });

    let cap_ph = t_display!(i18n, captcha_placeholder).to_string();

    view! {
        <div class="space-y-3 border-t pt-4 mt-4">
            <label class="form-label">{move || t!(i18n, captcha_label)}</label>
            <div class="flex items-center gap-2">
                <div class="rounded overflow-hidden cursor-pointer shrink-0"
                     style="width:160px;height:40px;border:1px solid #d1d5db"
                     inner_html=svg
                     on:click=move |_| captcha_res.refetch() />
                <input type="text" name="captcha_answer" required node_ref=answer_ref
                       placeholder=cap_ph
                       class="form-input w-16 text-center text-xl" on:input=on_input
                       prop:value=move || state.answer.get() autocomplete="off" />
                <button type="button"
                        class="text-blue-500 hover:text-blue-700 text-lg font-bold shrink-0 leading-none"
                        on:click=move |_| captcha_res.refetch() >
                    "↻"
                </button>
                <span class=move || if state.ok.get() { "text-green-500 font-bold text-sm" }
                                     else if status_msg.get().is_empty() { "text-gray-300 text-sm" }
                                     else { "text-red-400 text-sm" }>
                    {move || status_msg.get()}
                </span>
            </div>
            <input type="hidden" name="captcha_token" value=move || state.token.get() />
        </div>
    }
}
