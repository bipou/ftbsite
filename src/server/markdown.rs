use pulldown_cmark::{html, Options, Parser};

/// 将 Markdown 渲染为 HTML（纯函数，服务端调用）
pub fn render_md(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(md, opts));
    out
}
