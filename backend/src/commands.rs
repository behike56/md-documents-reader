use crate::{content::load_document, document::Document};
use tauri::{Manager, path::BaseDirectory};

#[tauri::command]
pub fn get_document(app: tauri::AppHandle, path: String) -> Result<Document, String> {
    let content_root = app
        .path()
        .resolve("content", BaseDirectory::Resource)
        .map_err(|error| error.to_string())?;
    load_document(&content_root, &path)
}
