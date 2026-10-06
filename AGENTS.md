# AGENTS.md

## 対象

- このファイルはリポジトリ全体に適用する。
- Rust バックエンドクレートは `backend/` にある。
- Svelteフロントエンドは `frontend/` にある。

## 作業方針

- 回答と作業結果は日本語で簡潔に報告する。
- ユーザーの未コミット変更を保持し、依頼と無関係なファイルを変更しない。
- 変更前に関連ファイルを確認し、推測ではなくリポジトリの実装を根拠にする。
- 実装した内容、検証済みの内容、未確認の内容を区別して報告する。

## 確認コマンド

変更後は、リポジトリルートから次のコマンドを実行する。

```bash
npm --prefix frontend run build
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
cargo check --manifest-path backend/Cargo.toml --all-targets --all-features
```

実行できない確認がある場合は、理由と未確認範囲を報告する。
