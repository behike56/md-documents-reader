use crate::document::{Document, sample_document};

#[tauri::command]
pub fn get_document() -> Document {
    sample_document()
}

#[cfg(test)]
mod tests {
    use super::get_document;

    #[test]
    fn returns_the_bundled_document() {
        let document = get_document();

        assert_eq!(document.title, "テスト用Markdown");
        assert!(document.html.contains("Markdown Reader PoC"));
    }
}
