# `md-documents-reader` 初期設計

## 文書の位置づけ

この文書は、TauriバックエンドとSvelteフロントエンドで構成するPoCの設計判断を記録する。実装済みの範囲と、PoC以降に決める事項を分けて扱う。

## 確認済みの現状

| 項目 | 現状 | 情報源 |
| --- | --- | --- |
| デスクトップ基盤 | Tauri 2 | `backend/Cargo.toml`、`backend/tauri.conf.json` |
| フロントエンド | Svelte 5、Vite | `frontend/package.json`、`frontend/src/App.svelte` |
| バックエンド | Rust 2024 Edition | `backend/Cargo.toml` |
| 実装済みの動作 | 同梱MarkdownをHTMLへ変換し、Tauriコマンド経由で画面に表示 | `backend/src/`、`frontend/src/App.svelte` |
| 品質チェック | Svelteビルド、`rustfmt`、Clippy、`cargo check` | `Makefile`、`.github/workflows/static-checks.yml` |

## PoCの対象範囲

PoCは、同梱した単一のMarkdown文書をデスクトップ画面へ表示するところまでを対象とする。

- 入力は `backend/content/sample.md` に固定する。
- Markdown変換は見出し、段落、箇条書き、コードブロックを扱う。
- 生HTMLは実行せず、Rust側でHTML特殊文字をエスケープする。
- 文書選択、編集、保存、検索、外部URL取得は対象外とする。
- CommonMarkおよびGitHub Flavored Markdownへの完全準拠は対象外とする。

## 構成と責務

```text
[Svelte UI]
    |
    | invoke("get_document")
    v
[Tauri command: commands.rs]
    |
    v
[Markdown処理: document.rs]
    |
    v
[同梱文書: content/sample.md]
```

| 領域 | 責務 |
| --- | --- |
| `frontend/` | 読み込み状態、エラー、変換済み文書を表示する |
| `backend/src/commands.rs` | Tauri IPCの境界としてユースケースを公開する |
| `backend/src/document.rs` | 同梱文書の取得、Markdown変換、HTMLエスケープを行う |
| `backend/src/lib.rs` | Tauriアプリを構成し、公開コマンドを登録する |

文書処理をTauriの起動処理やUIから分離し、単体テスト可能な関数として保つ。差し替える実装が複数になるまでは、トレイトなどの抽象化を追加しない。

## 実装した処理フロー

1. Svelteコンポーネントのマウント後に `get_document` を呼び出す。
2. TauriがRustの `commands::get_document` へ要求を渡す。
3. Rustが同梱Markdownを安全なHTMLへ変換する。
4. Tauriが `title` と `html` をフロントエンドへ返す。
5. Svelteがタイトルと本文を画面に表示する。

HTTPサーバーや外部公開APIは使用しない。フロントエンドとバックエンドの境界はTauri IPCに限定する。

## セキュリティ方針

- Markdown中の `&`、`<`、`>`、引用符をHTMLエンティティへ変換する。
- UIが `{@html}` で描画する値は、Rust側で生成したHTMLだけに限定する。
- Content Security Policyは同梱リソースを基本とし、外部スクリプトを許可しない。
- 将来ファイル選択を追加する場合は、Tauri capabilityと許可するパス範囲を別途設計する。

## 検証方針

| 対象 | 検証 |
| --- | --- |
| Markdown処理 | 対応ブロックの変換と生HTMLのエスケープをRust単体テストで確認する |
| Tauri境界 | コマンドが同梱文書を返すことをRust単体テストで確認する |
| Svelte UI | Viteのプロダクションビルドが成功することを確認する |
| 結合動作 | Tauriデスクトップアプリを起動し、テスト文書が表示されることを確認する |
| 静的品質 | `rustfmt`、Clippy、`cargo check` を実行する |

## 設計判断とトレードオフ

| 判断 | ステータス | 利点 | コスト・制約 |
| --- | --- | --- | --- |
| バックエンドにTauri 2を採用する | 実装済み | Web技術のUIとRust処理をデスクトップアプリとして統合できる | OSごとのビルド依存関係が必要になる |
| フロントエンドにSvelte 5とViteを採用する | 実装済み | UIの状態と表示を小さなコンポーネントで表現できる | Node.jsのビルド工程が増える |
| Tauriコマンドで文書を取得する | 実装済み | ローカルHTTPサーバーとポート管理が不要になる | 境界がTauri APIに依存する |
| Markdown変換をRust側に置く | 実装済み | 入力の安全化を一箇所に集約できる | 対応構文を増やす場合はパーサー導入の検討が必要になる |
| 単一文書から始める | 実装済み | 最小のユースケースで全体の接続を検証できる | 文書選択やライブラリ管理は未実装となる |

## 要確認事項

PoC以降は、次を決める必要がある。

1. 利用者が任意のローカルファイルを選択できるようにするか。
2. CommonMarkまたはGitHub Flavored Markdownのどこまでを対象にするか。
3. 生HTML、画像、外部リンクを許可するか。
4. 文書の編集・保存・検索を対象にするか。
5. 1文書あたりの想定サイズと応答性能をどう定めるか。
