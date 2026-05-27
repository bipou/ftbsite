use leptos::either::Either;
use leptos::html::{Input, Textarea};
use leptos::prelude::*;

use crate::i18n::{t, use_i18n};
use crate::shared::fns::{GetUploadNonce, PreviewMd, UploadImage};

#[component]
pub fn MarkdownEditor(
    name: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(default = 15)] rows: u32,
    #[prop(into, default = "user".to_string())] scope: String,
) -> impl IntoView {
    let i18n = use_i18n();
    let (markdown, set_markdown) = signal(value.get());
    let (show_preview, set_show_preview) = signal(false);
    let textarea_ref = NodeRef::<Textarea>::new();
    let file_input_ref = NodeRef::<Input>::new();

    // 仅在外部 value 变化时同步（回填编辑中的内容），不追踪 markdown
    Effect::new(move |_| {
        let changed = value.with(|v| markdown.with_untracked(|m| v != m));
        if changed {
            set_markdown.set(value.get());
        }
    });

    // 预取上传 nonce（组件挂载时请求一次，30 分钟有效）
    let nonce_action = ServerAction::<GetUploadNonce>::new();
    Effect::new(move |_| {
        nonce_action.dispatch(GetUploadNonce {});
    });

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

    // 监听 trigger + nonce，两者都就绪才 dispatch
    Effect::new(move |_| {
        let trigger = upload_trigger.get();
        let n = nonce_action.value().get().and_then(|r| r.ok());
        if let (Some((data_url, filename)), Some(n)) = (trigger, n.filter(|n| !n.is_empty())) {
            upload_action.dispatch(UploadImage {
                data_url,
                _filename: filename,
                nonce: n,
                scope: scope.clone(),
            });
            set_upload_trigger.set(None);
        }
    });

    // 记录插入图片时的光标位置
    let (cursor_pos, set_cursor_pos) = signal(usize::MAX);

    // 上传成功 → 在光标位置插入图片语法
    Effect::new(move |_| {
        if let Some(Ok(url)) = upload_action.value().get() {
            let cursor = cursor_pos.get_untracked();
            let md = format!("![图片]({url})");
            set_markdown.update(|v| {
                if cursor < v.len() {
                    // 在光标位置插入
                    let prefix: String = v.chars().take(cursor).collect();
                    let suffix: String = v.chars().skip(cursor).collect();
                    let mut new_v = prefix;
                    if !new_v.is_empty() && !new_v.ends_with('\n') {
                        new_v.push('\n');
                    }
                    new_v.push_str(&md);
                    if !suffix.is_empty() && !suffix.starts_with('\n') {
                        new_v.push('\n');
                    }
                    new_v.push_str(&suffix);
                    *v = new_v;
                } else {
                    // 光标无效（如未点击按钮）→ 追加到末尾
                    if !v.is_empty() && !v.ends_with('\n') {
                        v.push('\n');
                    }
                    v.push_str(&md);
                }
            });
            set_cursor_pos.set(usize::MAX); // 重置
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
                <Show when=move || !show_preview.get()>
                    <button type="button"
                        class="cursor-pointer"
                        style="font-size:1.1rem;border:0;background:transparent;padding:0;line-height:1"
                        on:click=move |_| {
                            // 捕获当前光标位置
                            #[cfg(feature = "hydrate")]
                            if let Some(ta) = textarea_ref.get() {
                                let pos = ta.selection_start()
                                    .ok()
                                    .flatten()
                                    .unwrap_or(0) as usize;
                                set_cursor_pos.set(pos);
                            }
                            if let Some(input) = file_input_ref.get() {
                                input.click();
                            }
                        }>
                        "🖼️"
                    </button>
                </Show>
            </div>

            // 编辑模式
            <Show when=move || !show_preview.get() fallback=|| ()>
                <textarea
                    id=format!("md-ed-{name}")
                    rows=rows
                    class="form-input"
                    node_ref=textarea_ref
                    prop:value=move || markdown.get()
                    on:input=move |ev| {
                        set_markdown.set(event_target_value(&ev));
                    }>
                </textarea>
            </Show>

            // 预览模式
            <Show when=move || show_preview.get() fallback=|| ()>
                <article class="card p-4 prose-sm"
                    inner_html=move || preview_html.get()>
                </article>
            </Show>

            // 表单隐藏域：始终提交最新 markdown 源码
            <input type="hidden" name=name value=move || markdown.get() />
        </div>
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

/// 读取用户选择的图片文件并转为 data URL，通过 set_trigger 通知上传流程
fn select_file(file_input: NodeRef<Input>, set_trigger: WriteSignal<Option<(String, String)>>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        use web_sys::{FileReader, HtmlInputElement};

        let Some(input) = file_input.get() else {
            return;
        };
        let html_el: &web_sys::HtmlElement = input.as_ref();
        let raw: &HtmlInputElement = html_el.unchecked_ref();
        let Some(files) = raw.files() else { return };
        let Some(file) = web_sys::FileList::item(&files, 0) else {
            return;
        };

        let filename = file.name();
        let reader = FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = wasm_bindgen::closure::Closure::once_into_js(Box::new(move || {
            if let Ok(result) = reader_clone.result() {
                if let Some(data_url) = result.as_string() {
                    set_trigger.set(Some((data_url, filename)));
                }
            }
        }));

        reader.set_onload(Some(onload.unchecked_ref()));
        let _ = reader.read_as_data_url(&file);

        // 重置以便重复选择同一文件
        raw.set_value("");
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (file_input, set_trigger);
    }
}
