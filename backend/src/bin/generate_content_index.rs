fn main() {
    if let Err(error) = md_documents_reader::generate_content_index() {
        eprintln!("コンテンツ一覧の生成に失敗しました: {error}");
        std::process::exit(1);
    }
}
