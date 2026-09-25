use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

const INDEX_HTML: &str = include_str!("../static/index.html");
const SAMPLE_MARKDOWN: &str = include_str!("../content/sample.md");
const MAX_REQUEST_SIZE: u64 = 16 * 1024;

pub fn run(address: &str) -> io::Result<()> {
    let listener = TcpListener::bind(address)?;
    println!("Markdown Reader: http://{address}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream) {
                    eprintln!("リクエストの処理に失敗しました: {error}");
                }
            }
            Err(error) => eprintln!("接続を受け付けられませんでした: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let request = read_request(stream)?;
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let response = response_for(path);
    stream.write_all(&response)
}

fn read_request(stream: &mut TcpStream) -> io::Result<String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];

    while (request.len() as u64) < MAX_REQUEST_SIZE {
        let bytes_read = stream.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..bytes_read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&request).into_owned())
}

fn response_for(path: &str) -> Vec<u8> {
    match path {
        "/" => http_response("200 OK", "text/html; charset=utf-8", INDEX_HTML),
        "/api/document" => {
            let html = render_markdown(SAMPLE_MARKDOWN);
            let body = format!(
                "{{\"title\":\"テスト用Markdown\",\"html\":\"{}\"}}",
                escape_json(&html)
            );
            http_response("200 OK", "application/json; charset=utf-8", &body)
        }
        _ => http_response(
            "404 Not Found",
            "application/json; charset=utf-8",
            r#"{"error":"ページが見つかりません"}"#,
        ),
    }
}

fn http_response(status: &str, content_type: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
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

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::{render_markdown, response_for};

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

    #[test]
    fn returns_document_api_response() {
        let response = String::from_utf8(response_for("/api/document")).unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("application/json"));
        assert!(response.contains("テスト用Markdown"));
        assert!(response.contains("Markdown Reader PoC"));
    }

    #[test]
    fn returns_not_found_for_unknown_path() {
        let response = String::from_utf8(response_for("/unknown")).unwrap();

        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
    }
}
