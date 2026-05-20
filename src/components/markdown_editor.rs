use leptos::either::Either;
use leptos::html::{Input, Textarea};
use leptos::prelude::*;

use crate::i18n::{t, use_i18n};
use crate::utils::common::{PreviewMd, UploadImage, get_upload_nonce};

#[component]
pub fn MarkdownEditor(
    name: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(default = 15)] rows: u32,
) -> impl IntoView {
    let i18n = use_i18n();
    let (markdown, set_markdown) = signal(value.get());
    let (show_preview, set_show_preview) = signal(false);
    let textarea_ref = NodeRef::<Textarea>::new();
    let file_input_ref = NodeRef::<Input>::new();
    let textarea_id = RwSignal::new(format!("md-ed-{name}"));

    // 仅在外部 value 变化时同步（编辑已有内容回填），不追踪 markdown
    Effect::new(move |_| {
        let changed = value.with(|v| markdown.with_untracked(|m| v != m));
        if changed {
            set_markdown.set(value.get());
        }
    });

    // 获取上传 nonce（一次性，30分钟有效）
    let nonce_res = Resource::new(|| (), |_| async move { get_upload_nonce().await.ok() });
    let nonce = move || nonce_res.get().flatten().unwrap_or_default();

    // 预览
    let preview_action = ServerAction::<PreviewMd>::new();
    let preview_html = Signal::derive(move || {
        preview_action
            .value()
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let toggle_preview = move |_| {
        if !show_preview.get() {
            preview_action.dispatch(PreviewMd { md: markdown.get() });
        }
        set_show_preview.set(!show_preview.get());
    };

    // 上传
    let upload_action = ServerAction::<UploadImage>::new();
    let (upload_trigger, set_upload_trigger) = signal(None::<(String, String)>);

    Effect::new(move |_| {
        if let Some((data_url, filename)) = upload_trigger.get() {
            let n = nonce();
            if !n.is_empty() {
                upload_action.dispatch(UploadImage {
                    data_url,
                    _filename: filename,
                    nonce: n,
                });
            }
            set_upload_trigger.set(None);
        }
    });

    // 上传成功 → 在光标处插入 markdown 图片语法
    Effect::new(move |_| {
        if let Some(Ok(url)) = upload_action.value().get() {
            let md = format!("![图片]({url})");
            textarea_id.with(|id| insert_at_cursor(id, &md));
            if let Some(el) = textarea_ref.get() {
                set_markdown.set(el.value());
            }
        }
    });

    view! {
        <div class="md-editor">
            <div class="flex items-center gap-2 mb-2">
                <button type="button" class="btn btn-secondary text-xs"
                    on:click=toggle_preview>
                    {move || if show_preview.get() {
                        Either::Left(t!(i18n, md_source))
                    } else {
                        Either::Right(t!(i18n, md_preview))
                    }}
                </button>
                <div class="flex-1"></div>
                <input type="file" accept="image/*" class="hidden"
                    node_ref=file_input_ref
                    on:change=move |_| {
                        select_file(file_input_ref, set_upload_trigger);
                    }
                />
                <button type="button"
                    class="cursor-pointer"
                    style="font-size:1.1rem;border:0;background:transparent;padding:0;line-height:1"
                    on:click=move |_| {
                        if let Some(input) = file_input_ref.get() {
                            input.click();
                        }
                    }>
                    "🖼️"
                </button>
            </div>

            // 源码模式：显示 textarea
            <Show when=move || !show_preview.get() fallback=|| ()>
                <textarea
                    id=textarea_id
                    rows=rows
                    class="form-input"
                    node_ref=textarea_ref
                    prop:value=move || markdown.get()
                    on:input=move |ev| {
                        set_markdown.set(event_target_value(&ev));
                    }>
                </textarea>
            </Show>

            // 预览模式：显示渲染结果
            <Show when=move || show_preview.get() fallback=|| ()>
                <article class="card p-4 prose-sm"
                    inner_html=move || preview_html.get()>
                </article>
            </Show>

            // 始终提交 markdown 源码到表单
            <input type="hidden" name=name value=move || markdown.get() />
        </div>
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

fn insert_at_cursor(textarea_id: &str, text: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(el) = document().get_element_by_id(textarea_id) {
            if let Some(raw) = el.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                let start = raw.selection_start().ok().flatten().unwrap_or(0) as usize;
                let end = raw.selection_end().ok().flatten().unwrap_or(0) as usize;
                let mut v = raw.value();
                let s = start.min(v.len());
                let e = end.min(v.len());
                v.replace_range(s..e, text);
                raw.set_value(&v);
                let pos = (start + text.len()) as u32;
                raw.set_selection_start(Some(pos)).ok();
                raw.set_selection_end(Some(pos)).ok();
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (textarea_id, text);
    }
}

fn select_file(file_input: NodeRef<Input>, set_trigger: WriteSignal<Option<(String, String)>>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        use web_sys::{FileReader, HtmlInputElement};

        if let Some(input) = file_input.get() {
            let html_el: &web_sys::HtmlElement = input.as_ref();
            let raw: &HtmlInputElement = html_el.unchecked_ref();
            if let Some(files) = raw.files() {
                if let Some(file) = web_sys::FileList::item(&files, 0) {
                    let filename = file.name();
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();

                    let onload =
                        wasm_bindgen::closure::Closure::once_into_js(Box::new(move || {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(data_url) = result.as_string() {
                                    set_trigger.set(Some((data_url, filename)));
                                }
                            }
                        }));

                    reader.set_onload(Some(onload.unchecked_ref()));
                    let _ = reader.read_as_data_url(&file);
                }
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (file_input, set_trigger);
    }
}
