pub fn render(text: &str) -> String {
    let options = pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS;
    let parser = pulldown_cmark::Parser::new_ext(text, options).map(|e| match e {
        pulldown_cmark::Event::Html(t) | pulldown_cmark::Event::InlineHtml(t) => {
            pulldown_cmark::Event::Text(t)
        }
        // No remote images: opening a chat must never contact a tracking server.
        pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image { .. }) => {
            pulldown_cmark::Event::Text("[Bild: ".into())
        }
        pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Image) => {
            pulldown_cmark::Event::Text("]".into())
        }
        x => x,
    });
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    ammonia::Builder::default()
        .url_schemes(std::collections::HashSet::from(["https", "http", "mailto"]))
        .link_rel(Some("noopener noreferrer"))
        .clean(&html)
        .to_string()
}
#[cfg(test)]
mod tests {
    #[test]
    fn safe_markdown() {
        let h = super::render(
            "# Titel\n\n```rust\nlet x = 1;\n```\n<script>alert(1)</script>\n![x](https://evil.test/a)\n[x](javascript:alert(1))",
        );
        assert!(h.contains("<pre><code"));
        assert!(!h.contains("<script>"));
        assert!(!h.contains("<img"));
        assert!(!h.contains("href=\"javascript:"));
    }
}
