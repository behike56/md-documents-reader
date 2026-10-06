use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Serialize;

use crate::document::{Categories, Document, parse_document};

const SOURCE_CONTENT_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/content");
const INDEX_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../frontend/src/generated/content-index.json"
);

#[derive(Serialize)]
struct ContentIndex {
    schema_version: u32,
    entries: Vec<ContentEntry>,
}

#[derive(Serialize)]
pub struct ContentEntry {
    name: String,
    path: String,
    kind: ContentKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Categories>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    children: Vec<ContentEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum ContentKind {
    Directory,
    Markdown,
}

fn list_content() -> Result<Vec<ContentEntry>, String> {
    scan_directory(
        Path::new(SOURCE_CONTENT_ROOT),
        Path::new(""),
        &mut BTreeMap::new(),
    )
}

pub fn write_content_index() -> Result<(), String> {
    let index = ContentIndex {
        schema_version: 2,
        entries: list_content()?,
    };
    let mut json = serde_json::to_string_pretty(&index).map_err(|error| error.to_string())?;
    json.push('\n');
    let output = Path::new(INDEX_PATH);
    if fs::read_to_string(output).is_ok_and(|existing| existing == json) {
        return Ok(());
    }
    fs::create_dir_all(output.parent().ok_or("出力先のディレクトリがありません")?)
        .map_err(|error| error.to_string())?;
    fs::write(output, json).map_err(|error| error.to_string())
}

fn scan_directory(
    directory: &Path,
    relative: &Path,
    seen_pages: &mut BTreeMap<(Categories, u32), String>,
) -> Result<Vec<ContentEntry>, String> {
    let mut entries = Vec::new();
    for item in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let item = item.map_err(|error| error.to_string())?;
        let file_type = item.file_type().map_err(|error| error.to_string())?;
        // Do not traverse symlinks outside the content directory.
        if file_type.is_symlink() {
            continue;
        }
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| "UTF-8ではないファイル名があります")?;
        let path = relative.join(&name);
        let (kind, title, categories, page, tags, description, children) = if file_type.is_dir() {
            let children = scan_directory(&item.path(), &path, seen_pages)?;
            if children.is_empty() {
                continue;
            }
            (
                ContentKind::Directory,
                None,
                None,
                None,
                Vec::new(),
                None,
                children,
            )
        } else if file_type.is_file() && is_markdown(&path) {
            let source = fs::read_to_string(item.path())
                .map_err(|error| format!("{}: {error}", path.display()))?;
            let document =
                parse_document(&source).map_err(|error| format!("{}: {error}", path.display()))?;
            let categories = document.categories.ok_or_else(|| {
                format!(
                    "{}: 一覧に載せる文書にはv2のcategoriesが必要です",
                    path.display()
                )
            })?;
            let page = document.page.ok_or_else(|| {
                format!("{}: 一覧に載せる文書にはv2のpageが必要です", path.display())
            })?;
            let display_path = path.to_string_lossy().replace('\\', "/");
            if let Some(previous) =
                seen_pages.insert((categories.clone(), page), display_path.clone())
            {
                return Err(format!(
                    "{display_path}: page {page} は同じ小カテゴリの {previous} と重複しています"
                ));
            }
            (
                ContentKind::Markdown,
                Some(document.title),
                Some(categories),
                Some(page),
                document.tags,
                document.description,
                Vec::new(),
            )
        } else {
            continue;
        };
        entries.push(ContentEntry {
            name,
            path: path.to_string_lossy().replace('\\', "/"),
            kind,
            title,
            categories,
            page,
            tags,
            description,
            children,
        });
    }
    entries.sort_by(|left, right| {
        matches!(right.kind, ContentKind::Directory)
            .cmp(&matches!(left.kind, ContentKind::Directory))
            .then_with(|| left.categories.cmp(&right.categories))
            .then_with(|| left.page.cmp(&right.page))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

pub fn load_document(content_root: &Path, relative_path: &str) -> Result<Document, String> {
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        || !is_markdown(relative)
    {
        return Err("無効なMarkdownファイルのパスです".to_owned());
    }

    let mut path = PathBuf::from(content_root);
    for part in relative.components() {
        path.push(part);
        if fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("シンボリックリンクは読み込めません".to_owned());
        }
    }
    let source = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    parse_document(&source).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        ContentIndex, ContentKind, SOURCE_CONTENT_ROOT, list_content, load_document, scan_directory,
    };

    #[test]
    fn represents_only_markdown_files_in_nested_directories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("md-reader-tree-{}-{unique}", std::process::id()));
        fs::create_dir_all(root.join("chapter")).unwrap();
        fs::write(
            root.join("chapter/guide.md"),
            include_str!("../content/sample.md"),
        )
        .unwrap();
        fs::write(root.join("notes.txt"), "Notes").unwrap();
        fs::write(
            root.join("intro.md"),
            include_str!("../content/sample.md").replace("page: 1", "page: 2"),
        )
        .unwrap();
        fs::create_dir(root.join("empty")).unwrap();

        let entries =
            scan_directory(&root, std::path::Path::new(""), &mut BTreeMap::new()).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(matches!(entries[0].kind, ContentKind::Directory));
        assert_eq!(entries[0].path, "chapter");
        assert_eq!(entries[0].children[0].path, "chapter/guide.md");
        assert!(matches!(entries[0].children[0].kind, ContentKind::Markdown));
        assert_eq!(
            entries[0].children[0].title.as_deref(),
            Some("Markdownテーマ設計")
        );
        assert_eq!(entries[1].path, "intro.md");
        assert_eq!(entries[1].page, Some(2));

        let json = serde_json::to_value(ContentIndex {
            schema_version: 2,
            entries,
        })
        .unwrap();
        assert_eq!(
            json["entries"][0]["children"][0]["path"],
            "chapter/guide.md"
        );
        assert_eq!(
            json["entries"][0]["children"][0]["categories"]["small"],
            "Markdown"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_duplicate_pages_within_the_same_small_category() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("md-reader-pages-{}-{unique}", std::process::id()));
        fs::create_dir_all(root.join("chapter")).unwrap();
        fs::write(root.join("first.md"), include_str!("../content/sample.md")).unwrap();
        fs::write(
            root.join("chapter/second.md"),
            include_str!("../content/sample.md"),
        )
        .unwrap();

        let error = scan_directory(&root, std::path::Path::new(""), &mut BTreeMap::new())
            .err()
            .unwrap();
        assert!(error.contains("page 1"));
        assert!(error.contains("first.md"));
        assert!(error.contains("second.md"));

        fs::write(
            root.join("chapter/second.md"),
            include_str!("../content/sample.md").replace("small: Markdown", "small: ' Markdown '"),
        )
        .unwrap();
        assert!(scan_directory(&root, std::path::Path::new(""), &mut BTreeMap::new()).is_err());

        fs::write(
            root.join("chapter/second.md"),
            include_str!("../content/sample.md").replace("small: Markdown", "small: 別カテゴリ"),
        )
        .unwrap();
        assert!(scan_directory(&root, std::path::Path::new(""), &mut BTreeMap::new()).is_ok());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lists_and_loads_the_sample() {
        let entries = list_content().unwrap();
        assert!(entries.iter().any(|entry| entry.path == "sample.md"));
        assert_eq!(
            load_document(std::path::Path::new(SOURCE_CONTENT_ROOT), "sample.md")
                .unwrap()
                .title,
            "Markdownテーマ設計"
        );
    }

    #[test]
    fn rejects_paths_outside_content_and_non_markdown_files() {
        for path in [
            "",
            "../Cargo.toml",
            "/tmp/example.md",
            "sample.md/../sample.md",
            "image.png",
        ] {
            assert!(
                load_document(std::path::Path::new(SOURCE_CONTENT_ROOT), path).is_err(),
                "{path}"
            );
        }
    }

    #[test]
    fn loads_document_from_a_moved_content_directory() {
        let root = std::env::temp_dir().join(format!("md-reader-bundle-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("sample.md"), include_str!("../content/sample.md")).unwrap();

        assert_eq!(
            load_document(&root, "sample.md").unwrap().title,
            "Markdownテーマ設計"
        );

        fs::remove_dir_all(root).unwrap();
    }
}
