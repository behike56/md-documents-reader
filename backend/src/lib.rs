mod commands;
mod document;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_document])
        .run(tauri::generate_context!())
        .expect("Tauriアプリケーションの実行に失敗しました");
}
