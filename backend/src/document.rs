use std::{error::Error, fmt};

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use serde::Deserialize;

const FORMAT_VERSION: u32 = 1;
const SAMPLE_MARKDOWN: &str = include_str!("../content/sample.md");

#[derive(Debug, Default, Deserialize, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTheme {
    #[default]
    Technical,
    Editorial,
}

#[derive(Debug, serde::Serialize)]
pub struct Document {
    pub title: String,
    pub theme: DocumentTheme,
    pub html: String,
}

#[derive(Debug, PartialEq)]
pub enum DocumentError {
    MissingFrontMatter,
    UnclosedFrontMatter,
    InvalidFrontMatter(String),
    UnsupportedFormatVersion(u32),
    EmptyTitle,
    LevelOneHeading,
    NestedAdmonition,
    UnclosedAdmonition,
    UnexpectedAdmonitionEnd,
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontMatter => {
                write!(formatter, "MarkdownファイルにFront Matterがありません")
            }
            Self::UnclosedFrontMatter => {
                write!(
                    formatter,
                    "MarkdownファイルのFront Matterが閉じられていません"
                )
            }
            Self::InvalidFrontMatter(error) => {
                write!(formatter, "Front Matterが不正です: {error}")
            }
            Self::UnsupportedFormatVersion(version) => {
                write!(formatter, "未対応のformat_versionです: {version}")
            }
            Self::EmptyTitle => write!(formatter, "Front Matterのtitleが空です"),
            Self::LevelOneHeading => {
                write!(
                    formatter,
                    "本文ではH1を使用できません。titleを使用してください"
                )
            }
            Self::NestedAdmonition => write!(formatter, "注意ブロックは入れ子にできません"),
            Self::UnclosedAdmonition => write!(formatter, "注意ブロックが閉じられていません"),
            Self::UnexpectedAdmonitionEnd => {
                write!(formatter, "対応する開始行のない注意ブロック終端です")
            }
        }
    }
}

impl Error for DocumentError {}

pub fn sample_document() -> Result<Document, DocumentError> {
    parse_document(SAMPLE_MARKDOWN)
}

pub fn parse_document(markdown: &str) -> Result<Document, DocumentError> {
    let (metadata_source, body) = split_front_matter(markdown)?;
    let metadata: DocumentMetadata = yaml_serde::from_str(metadata_source)
        .map_err(|error| DocumentError::InvalidFrontMatter(error.to_string()))?;

    if metadata.format_version != FORMAT_VERSION {
        return Err(DocumentError::UnsupportedFormatVersion(
            metadata.format_version,
        ));
    }

    let title = metadata.title.trim();
    if title.is_empty() {
        return Err(DocumentError::EmptyTitle);
    }

    let body_html = render_markdown(body)?;
    let mut document_html = String::with_capacity(title.len() + body_html.len() + 10);
    document_html.push_str("<h1>");
    document_html.push_str(&escape_html(title));
    document_html.push_str("</h1>\n");
    document_html.push_str(&body_html);

    Ok(Document {
        title: title.to_owned(),
        theme: metadata.theme,
        html: document_html,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentMetadata {
    format_version: u32,
    title: String,
    #[serde(default)]
    theme: DocumentTheme,
}

fn split_front_matter(markdown: &str) -> Result<(&str, &str), DocumentError> {
    let mut lines = markdown.split_inclusive('\n');
    let Some(opening) = lines.next() else {
        return Err(DocumentError::MissingFrontMatter);
    };

    if trim_line_ending(opening) != "---" {
        return Err(DocumentError::MissingFrontMatter);
    }

    let metadata_start = opening.len();
    let mut offset = metadata_start;

    for line in lines {
        if trim_line_ending(line) == "---" {
            let body_start = offset + line.len();
            return Ok((&markdown[metadata_start..offset], &markdown[body_start..]));
        }
        offset += line.len();
    }

    Err(DocumentError::UnclosedFrontMatter)
}

fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line)
}

fn render_markdown(markdown: &str) -> Result<String, DocumentError> {
    let markdown = isolate_admonition_markers(markdown);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&markdown, options);
    let events = transform_events(parser)?;
    let mut rendered = String::new();
    html::push_html(&mut rendered, events.into_iter());
    Ok(rendered)
}

#[derive(Clone, Copy)]
enum AdmonitionMarker {
    Start(&'static str),
    End,
}

fn transform_events<'a>(
    events: impl IntoIterator<Item = Event<'a>>,
) -> Result<Vec<Event<'a>>, DocumentError> {
    let events: Vec<_> = events.into_iter().collect();
    let mut transformed = Vec::new();
    let mut admonition_open = false;
    let mut suppressed_link = false;
    let mut index = 0;

    while index < events.len() {
        if let Some(marker) = standalone_admonition_marker(&events, index) {
            match marker {
                AdmonitionMarker::Start(kind) => {
                    if admonition_open {
                        return Err(DocumentError::NestedAdmonition);
                    }
                    transformed.push(Event::Html(CowStr::Boxed(
                        format!("<aside class=\"admonition admonition-{kind}\">\n")
                            .into_boxed_str(),
                    )));
                    admonition_open = true;
                }
                AdmonitionMarker::End => {
                    if !admonition_open {
                        return Err(DocumentError::UnexpectedAdmonitionEnd);
                    }
                    transformed.push(Event::Html(CowStr::Borrowed("</aside>\n")));
                    admonition_open = false;
                }
            }
            index += 3;
            continue;
        }

        let event = events[index].clone();
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => return Err(DocumentError::LevelOneHeading),
            Event::Html(content) | Event::InlineHtml(content) => {
                transformed.push(Event::Text(content));
            }
            Event::Start(Tag::Link { dest_url, .. }) if !is_safe_link(&dest_url) => {
                suppressed_link = true;
            }
            Event::End(TagEnd::Link) if suppressed_link => {
                suppressed_link = false;
            }
            Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image) => {}
            event => transformed.push(event),
        }
        index += 1;
    }

    if admonition_open {
        return Err(DocumentError::UnclosedAdmonition);
    }

    Ok(transformed)
}

fn standalone_admonition_marker(events: &[Event<'_>], index: usize) -> Option<AdmonitionMarker> {
    let [
        Event::Start(Tag::Paragraph),
        Event::Text(content),
        Event::End(TagEnd::Paragraph),
    ] = events.get(index..index + 3)?
    else {
        return None;
    };

    match content.as_ref() {
        ":::info" => Some(AdmonitionMarker::Start("info")),
        ":::warning" => Some(AdmonitionMarker::Start("warning")),
        ":::success" => Some(AdmonitionMarker::Start("success")),
        ":::" => Some(AdmonitionMarker::End),
        _ => None,
    }
}

fn isolate_admonition_markers(markdown: &str) -> String {
    let mut isolated = String::with_capacity(markdown.len());
    let mut fence = None;

    for line in markdown.lines() {
        let marker = fenced_code_marker(line);
        let outside_code = fence.is_none();

        if outside_code && is_admonition_line(line) {
            if !isolated.is_empty() && !isolated.ends_with("\n\n") {
                isolated.push('\n');
            }
            isolated.push_str(line);
            isolated.push_str("\n\n");
        } else {
            isolated.push_str(line);
            isolated.push('\n');
        }

        match (fence, marker) {
            (None, Some((kind, length, remainder))) if kind == '~' || !remainder.contains('`') => {
                fence = Some((kind, length));
            }
            (Some((open_kind, open_length)), Some((kind, length, remainder)))
                if kind == open_kind && length >= open_length && remainder.trim().is_empty() =>
            {
                fence = None;
            }
            _ => {}
        }
    }

    isolated
}

fn is_admonition_line(line: &str) -> bool {
    matches!(line, ":::info" | ":::warning" | ":::success" | ":::")
}

fn fenced_code_marker(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }

    let kind = trimmed.chars().next()?;
    if !matches!(kind, '`' | '~') {
        return None;
    }

    let length = trimmed
        .chars()
        .take_while(|character| *character == kind)
        .count();
    if length < 3 {
        return None;
    }

    Some((kind, length, &trimmed[length..]))
}

fn is_safe_link(destination: &str) -> bool {
    let destination = destination.trim();
    if destination.starts_with('#')
        || destination.starts_with('/')
        || destination.starts_with("./")
        || destination.starts_with("../")
    {
        return true;
    }

    let Some((scheme, _)) = destination.split_once(':') else {
        return true;
    };

    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "mailto"
    )
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
    use super::{DocumentError, DocumentTheme, parse_document, sample_document};

    fn markdown(front_matter: &str, body: &str) -> String {
        format!("---\n{front_matter}\n---\n\n{body}")
    }

    #[test]
    fn renders_a_valid_version_one_document() {
        let source = markdown(
            "format_version: 1\ntitle: 設計ノート\ntheme: editorial",
            "## 本文\n\n**重要**です。",
        );
        let document = parse_document(&source).unwrap();

        assert_eq!(document.title, "設計ノート");
        assert_eq!(document.theme, DocumentTheme::Editorial);
        assert!(document.html.starts_with("<h1>設計ノート</h1>"));
        assert!(document.html.contains("<h2>本文</h2>"));
        assert!(document.html.contains("<strong>重要</strong>"));
    }

    #[test]
    fn uses_the_technical_theme_when_theme_is_omitted() {
        let source = markdown("format_version: 1\ntitle: 設計ノート", "本文");
        let document = parse_document(&source).unwrap();

        assert_eq!(document.theme, DocumentTheme::Technical);
    }

    #[test]
    fn rejects_markdown_without_front_matter() {
        for source in ["", "## 本文", "\u{feff}---\nformat_version: 1\n---"] {
            let error = parse_document(source).unwrap_err();
            assert_eq!(error, DocumentError::MissingFrontMatter);
        }
    }

    #[test]
    fn rejects_unclosed_front_matter() {
        let error = parse_document("---\nformat_version: 1\ntitle: 設計ノート").unwrap_err();

        assert_eq!(error, DocumentError::UnclosedFrontMatter);
    }

    #[test]
    fn rejects_invalid_front_matter_fields() {
        let cases = [
            "format_version: 1",
            "title: 設計ノート",
            "format_version: 1\ntitle: 設計ノート\ntheme: neon",
            "format_version: 1\ntitle: 設計ノート\nunknown: value",
            "format_version: 1\ntitle: 設計ノート\ntitle: 重複",
        ];

        for front_matter in cases {
            let error = parse_document(&markdown(front_matter, "本文")).unwrap_err();
            assert!(matches!(error, DocumentError::InvalidFrontMatter(_)));
        }
    }

    #[test]
    fn rejects_an_unsupported_format_version() {
        let source = markdown("format_version: 2\ntitle: 設計ノート", "本文");
        let error = parse_document(&source).unwrap_err();

        assert_eq!(error, DocumentError::UnsupportedFormatVersion(2));
    }

    #[test]
    fn rejects_an_empty_title() {
        let source = markdown("format_version: 1\ntitle: '   '", "本文");
        let error = parse_document(&source).unwrap_err();

        assert_eq!(error, DocumentError::EmptyTitle);
    }

    #[test]
    fn rejects_a_level_one_heading_in_the_body() {
        let source = markdown("format_version: 1\ntitle: 設計ノート", "# 重複タイトル");
        let error = parse_document(&source).unwrap_err();

        assert_eq!(error, DocumentError::LevelOneHeading);
    }

    #[test]
    fn renders_selected_gfm_extensions() {
        let source = markdown(
            "format_version: 1\ntitle: GFM",
            "~~削除~~\n\n- [x] 完了\n\n| 項目 | 値 |\n| --- | --- |\n| A | 1 |",
        );
        let document = parse_document(&source).unwrap();

        assert!(document.html.contains("<del>削除</del>"));
        assert!(document.html.contains("type=\"checkbox\""));
        assert!(document.html.contains("<table>"));
    }

    #[test]
    fn escapes_raw_html_and_removes_unsafe_links_and_images() {
        let source = markdown(
            "format_version: 1\ntitle: 安全性",
            "<script>alert('xss')</script>\n\n[危険](javascript:alert(1))\n\n![画像](image.png)",
        );
        let document = parse_document(&source).unwrap();

        assert!(document.html.contains("&lt;script&gt;"));
        assert!(!document.html.contains("<script>"));
        assert!(!document.html.contains("javascript:"), "{}", document.html);
        assert!(!document.html.contains("<img"));
        assert!(document.html.contains("危険"));
        assert!(document.html.contains("画像"));
    }

    #[test]
    fn preserves_safe_links_and_escapes_the_document_title() {
        let source = markdown(
            "format_version: 1\ntitle: \"<設計 & 検証>\"",
            "[外部](https://example.com) [内部](./guide.md)",
        );
        let document = parse_document(&source).unwrap();

        assert!(
            document
                .html
                .starts_with("<h1>&lt;設計 &amp; 検証&gt;</h1>")
        );
        assert!(document.html.contains("href=\"https://example.com\""));
        assert!(document.html.contains("href=\"./guide.md\""));
    }

    #[test]
    fn accepts_crlf_line_endings() {
        let source = "---\r\nformat_version: 1\r\ntitle: CRLF\r\n---\r\n\r\n## 本文\r\n";
        let document = parse_document(source).unwrap();

        assert_eq!(document.title, "CRLF");
        assert!(document.html.contains("<h2>本文</h2>"));
    }

    #[test]
    fn renders_markdown_inside_an_admonition() {
        let source = markdown(
            "format_version: 1\ntitle: 注意",
            ":::warning\n**元に戻せません。**\n:::",
        );
        let document = parse_document(&source).unwrap();

        assert!(document.html.contains(
            "<aside class=\"admonition admonition-warning\">\n<p><strong>元に戻せません。</strong></p>\n</aside>"
        ));
    }

    #[test]
    fn rejects_invalid_admonition_boundaries() {
        let cases = [
            (
                ":::warning\n閉じていません",
                DocumentError::UnclosedAdmonition,
            ),
            (":::", DocumentError::UnexpectedAdmonitionEnd),
            (
                ":::warning\n:::info\n入れ子\n:::\n:::",
                DocumentError::NestedAdmonition,
            ),
        ];

        for (body, expected) in cases {
            let source = markdown("format_version: 1\ntitle: 注意", body);
            assert_eq!(parse_document(&source).unwrap_err(), expected);
        }
    }

    #[test]
    fn treats_admonition_markers_inside_code_fences_as_code() {
        let source = markdown(
            "format_version: 1\ntitle: コード",
            "```text\n:::warning\n:::\n```",
        );
        let document = parse_document(&source).unwrap();

        assert!(document.html.contains(":::warning"));
        assert!(!document.html.contains("<aside"));
    }

    #[test]
    fn bundled_document_conforms_to_version_one() {
        let document = sample_document().unwrap();

        assert_eq!(document.title, "Markdownテーマ設計");
        assert_eq!(document.theme, DocumentTheme::Technical);
        assert!(document.html.starts_with("<h1>Markdownテーマ設計</h1>"));
        assert!(!document.html.contains("theme: technical"));
    }
}
