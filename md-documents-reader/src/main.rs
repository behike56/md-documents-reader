use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let address = env::var("MD_READER_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());

    match md_documents_reader::run(&address) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("サーバーを起動できませんでした: {error}");
            ExitCode::FAILURE
        }
    }
}
