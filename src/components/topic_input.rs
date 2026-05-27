use crate::i18n::{t, use_i18n};
use leptos::html::Input;
use leptos::prelude::*;

/// 话题输入组件——注册页/发表页共用
#[component]
pub fn TopicInput(
    #[prop(into, default = RwSignal::new(String::new()))] csv_out: RwSignal<String>,
) -> impl IntoView {
    let (topics, set_topics) = signal(Vec::<String>::new());
    let (input, set_input) = signal(String::new());
    let input_ref = NodeRef::<Input>::new();

    let add = move |name: &str| {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return;
        }
        set_topics.update(|v| {
            if !v.contains(&name) {
                v.push(name);
            }
        });
        set_input.set(String::new());
    };

    let remove = move |i: usize| {
        set_topics.update(|v| {
            v.remove(i);
        });
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
        "Enter" | "," | " " => {
            ev.prevent_default();
            add(&input.get());
        }
        "Backspace" => {
            if input.get().is_empty() {
                set_topics.update(|v| {
                    v.pop();
                });
            }
        }
        _ => {}
    };

    let csv = move || {
        let v = topics.get().join(",");
        csv_out.set(v.clone());
        v
    };

    view! {
        <label class="form-label">{move || t!(use_i18n(), input_topics)}</label>
        <div class="form-input flex flex-wrap items-center gap-1 cursor-text"
            on:click=move |_| {
                if let Some(el) = input_ref.get() {
                    let _ = el.focus();
                }
            }>
            {move || topics.get().iter().enumerate().map(|(i, t)| {
                let t = t.clone();
                view! {
                    <span class="badge-blue inline-flex items-center gap-1 text-xs">
                        {t}
                        <button type="button"
                            class="ml-0.5 text-blue-500 hover:text-red-500 font-bold leading-none cursor-pointer border-0 bg-transparent p-0 text-base"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                remove(i);
                            }>
                            "×"
                        </button>
                    </span>
                }
            }).collect::<Vec<_>>()}
            <input
                type="text"
                node_ref=input_ref
                class="border-0 outline-none flex-1 min-w-24 bg-transparent text-sm"
                placeholder="..."
                on:keydown=on_keydown
                on:input=move |ev| set_input.set(event_target_value(&ev))
                prop:value=input
            />
        </div>
        <input type="hidden" name="topics" prop:value=csv/>
    }
}
