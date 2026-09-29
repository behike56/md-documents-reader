const SAMPLE_MARKDOWN: &str = include_str!("../content/sample.md");

#[derive(Debug, serde::Serialize)]
pub struct Document {
    pub title: String,
    pub html: String,
}

pub fn sample_document() -> Document {
    Document {
        title: "テスト用Markdown".to_owned(),
        html: render_markdown(SAMPLE_MARKDOWN),
    }
}

pub fn render_markdown(markdown: &str) -> String {
    let mut html = String::new();
    let mut paragraph = Vec::new();
    let mut in_list = false;
    let mut in_code = false;

    for line in markdown.lines().chain(std::iter::once("")) {
        if line.trim_start().starts_with("```") {
            flush_paragraph(&mut html, &mut paragraph);
            close_list(&mut html, &mut in_list);
            if in_code {
                html.push_str("</code></pre>\n");
            } else {
                html.push_str("<pre><code>");
            }
            in_code = !in_code;
            continue;
        }

        if in_code {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        if let Some(item) = line.strip_prefix("- ") {
            flush_paragraph(&mut html, &mut paragraph);
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str("<li>");
            html.push_str(&escape_html(item));
            html.push_str("</li>\n");
            continue;
        }

        close_list(&mut html, &mut in_list);

        if line.is_empty() {
            flush_paragraph(&mut html, &mut paragraph);
        } else if let Some(heading) = line.strip_prefix("# ") {
            flush_paragraph(&mut html, &mut paragraph);
            html.push_str("<h1>");
            html.push_str(&escape_html(heading));
            html.push_str("</h1>\n");
        } else if let Some(heading) = line.strip_prefix("## ") {
            flush_paragraph(&mut html, &mut paragraph);
            html.push_str("<h2>");
            html.push_str(&escape_html(heading));
            html.push_str("</h2>\n");
        } else {
            paragraph.push(escape_html(line));
        }
    }

    if in_code {
        html.push_str("</code></pre>\n");
    }

    html
}

fn flush_paragraph(html: &mut String, paragraph: &mut Vec<String>) {
    if !paragraph.is_empty() {
        html.push_str("<p>");
        html.push_str(&paragraph.join(" "));
        html.push_str("</p>\n");
        paragraph.clear();
    }
}

fn close_list(html: &mut String, in_list: &mut bool) {
    if *in_list {
        html.push_str("</ul>\n");
        *in_list = false;
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::render_markdown;

    #[test]
    fn renders_supported_markdown_blocks() {
        let markdown = "# 見出し\n\n本文です。\n\n- 1つ目\n- 2つ目\n\n```rust\nlet x = 1;\n```";
        let html = render_markdown(markdown);

        assert!(html.contains("<h1>見出し</h1>"));
        assert!(html.contains("<p>本文です。</p>"));
        assert!(html.contains("<ul>\n<li>1つ目</li>\n<li>2つ目</li>\n</ul>"));
        assert!(html.contains("<pre><code>let x = 1;\n</code></pre>"));
    }

    #[test]
    fn escapes_raw_html() {
        let html = render_markdown("<script>alert('xss')</script>");

        assert_eq!(
            html,
            "<p>&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</p>\n"
        );
    }
}
