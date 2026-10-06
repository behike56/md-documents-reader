use std::{error::Error, fmt};

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use serde::Deserialize;

const FORMAT_VERSION: u32 = 2;

#[derive(Debug, Default, Deserialize, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTheme {
    #[default]
    Technical,
    Editorial,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Categories {
    pub large: String,
    pub medium: String,
    pub small: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Document {
    pub title: String,
    pub theme: DocumentTheme,
    pub categories: Option<Categories>,
    pub page: Option<u32>,
    pub tags: Vec<String>,
    pub description: Option<String>,
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

pub fn parse_document(markdown: &str) -> Result<Document, DocumentError> {
    let (metadata_source, body) = split_front_matter(markdown)?;
    let metadata: DocumentMetadata = yaml_serde::from_str(metadata_source)
        .map_err(|error| DocumentError::InvalidFrontMatter(error.to_string()))?;

    if !matches!(metadata.format_version, 1 | FORMAT_VERSION) {
        return Err(DocumentError::UnsupportedFormatVersion(
            metadata.format_version,
        ));
    }

    if metadata.format_version == 1
        && (metadata.categories.is_some()
            || metadata.page.is_some()
            || metadata.tags.is_some()
            || metadata.description.is_some())
    {
        return Err(DocumentError::InvalidFrontMatter(
            "v1ではcategories、page、tags、descriptionを指定できません".to_owned(),
        ));
    }
    let categories = metadata.categories.map(|mut categories| {
        categories.large = categories.large.trim().to_owned();
        categories.medium = categories.medium.trim().to_owned();
        categories.small = categories.small.trim().to_owned();
        categories
    });
    let tags = metadata.tags.unwrap_or_default();
    if metadata.format_version == FORMAT_VERSION
        && (categories.is_none() || metadata.page.is_none())
    {
        return Err(DocumentError::InvalidFrontMatter(
            "v2ではcategoriesとpageが必須です".to_owned(),
        ));
    }
    if categories.as_ref().is_some_and(|categories| {
        [&categories.large, &categories.medium, &categories.small]
            .iter()
            .any(|value| value.trim().is_empty())
    }) || tags.iter().any(|value| value.trim().is_empty())
    {
        return Err(DocumentError::InvalidFrontMatter(
            "categoriesとtagsに空の値は指定できません".to_owned(),
        ));
    }
    if metadata.page == Some(0) {
        return Err(DocumentError::InvalidFrontMatter(
            "pageには1以上の整数を指定してください".to_owned(),
        ));
    }
    if metadata
        .description
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(DocumentError::InvalidFrontMatter(
            "descriptionに空の値は指定できません".to_owned(),
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
        categories,
        page: metadata.page,
        tags,
        description: metadata.description,
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
    categories: Option<Categories>,
    page: Option<u32>,
    tags: Option<Vec<String>>,
    description: Option<String>,
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
    use super::{DocumentError, DocumentTheme, parse_document};

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
    fn reads_version_two_index_metadata() {
        let source = markdown(
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 3\ntags: [テーマ]\ndescription: 概要です。",
            "本文",
        );
        let document = parse_document(&source).unwrap();

        let categories = document.categories.unwrap();
        assert_eq!(categories.large, "設計");
        assert_eq!(categories.medium, "表示");
        assert_eq!(categories.small, "Markdown");
        assert_eq!(document.page, Some(3));
        assert_eq!(document.tags, ["テーマ"]);
        assert_eq!(document.description.as_deref(), Some("概要です。"));
    }

    #[test]
    fn rejects_missing_or_invalid_version_two_categories_and_page() {
        for front_matter in [
            "format_version: 2\ntitle: 設計ノート",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: '  '\npage: 1",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\npage: 1",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 0",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 1.5",
        ] {
            assert!(matches!(
                parse_document(&markdown(front_matter, "本文")),
                Err(DocumentError::InvalidFrontMatter(_))
            ));
        }
    }

    #[test]
    fn rejects_invalid_version_two_optional_metadata() {
        for front_matter in [
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 1\ntags: ['  ']",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 1\ndescription: '  '",
            "format_version: 2\ntitle: 設計ノート\ncategories:\n  large: 設計\n  medium: 表示\n  small: Markdown\npage: 1\ntags: 12",
            "format_version: 1\ntitle: 設計ノート\npage: 1",
        ] {
            assert!(matches!(
                parse_document(&markdown(front_matter, "本文")),
                Err(DocumentError::InvalidFrontMatter(_))
            ));
        }
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
        let source = markdown("format_version: 3\ntitle: 設計ノート", "本文");
        let error = parse_document(&source).unwrap_err();

        assert_eq!(error, DocumentError::UnsupportedFormatVersion(3));
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
    fn bundled_document_conforms_to_version_two() {
        let document = parse_document(include_str!("../content/sample.md")).unwrap();

        assert_eq!(document.title, "Markdownテーマ設計");
        assert_eq!(document.theme, DocumentTheme::Technical);
        let categories = document.categories.unwrap();
        assert_eq!(categories.large, "設計");
        assert_eq!(categories.medium, "表示");
        assert_eq!(categories.small, "Markdown");
        assert_eq!(document.page, Some(1));
        assert!(document.html.starts_with("<h1>Markdownテーマ設計</h1>"));
        assert!(!document.html.contains("theme: technical"));
    }
}
