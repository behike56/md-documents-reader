use crate::{content::load_document, document::Document};

#[tauri::command]
pub fn get_document(path: String) -> Result<Document, String> {
    load_document(&path)
}

#[cfg(test)]
mod tests {
    use crate::document::DocumentTheme;

    use super::get_document;

    #[test]
    fn returns_the_bundled_document() {
        let document = get_document("sample.md".to_owned()).unwrap();

        assert_eq!(document.title, "Markdownテーマ設計");
        assert_eq!(document.theme, DocumentTheme::Technical);
        assert!(document.html.contains("<h2>概要</h2>"));
    }
}
