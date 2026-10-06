# `md-documents-reader` 初期設計

## 文書の位置づけ

この文書は、TauriバックエンドとSvelteフロントエンドで構成するPoCの設計判断を記録する。実装済みの範囲と、PoC以降に決める事項を分けて扱う。

## 確認済みの現状

| 項目 | 現状 | 情報源 |
| --- | --- | --- |
| デスクトップ基盤 | Tauri 2 | `backend/Cargo.toml`、`backend/tauri.conf.json` |
| フロントエンド | Svelte 5、Vite | `frontend/package.json`、`frontend/src/App.svelte` |
| バックエンド | Rust 2024 Edition | `backend/Cargo.toml` |
| 実装済みの動作 | 同梱MarkdownをHTMLへ変換し、文書テーマとライト／ダーク表示を適用して画面に表示 | `backend/src/`、`frontend/src/` |
| 品質チェック | Svelteビルド、`rustfmt`、Clippy、`cargo check` | `Makefile`、`.github/workflows/static-checks.yml` |

## PoCの対象範囲

PoCは、同梱した単一のMarkdown文書をデスクトップ画面へ表示するところまでを対象とする。

- 入力は `backend/content/sample.md` に固定する。
- 入力は[Markdown Reader Format v1](./markdown-format-v1.md)に従う。
- YAML Front Matterは`format_version`と`title`を必須とし、`theme`を任意とする。
- 本文はCommonMarkを基礎とし、選択したGFM拡張と意味ベースの注意ブロックを扱う。
- 生HTMLは実行せず、Rust側でHTML特殊文字をエスケープする。
- 文書選択、編集、保存、検索、外部URL取得は対象外とする。
- H1、raw HTML、リンク、画像にはアプリケーション固有の制約を適用するため、GFM完全互換は対象外とする。

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
| `frontend/src/App.svelte` | アプリUIとライト／ダーク表示の状態を管理する |
| `frontend/src/styles/tokens.css` | アプリと文書表示が共有する色・影などのデザイントークンを定義する |
| `frontend/src/styles/markdown.css` | 変換済みMarkdownの見出し、表、引用、コード、注意ブロックを装飾する |
| `backend/src/commands.rs` | Tauri IPCの境界としてユースケースを公開する |
| `backend/src/document.rs` | YAML Front Matter検証、Markdown変換、アプリケーション固有規則の適用を行う |
| `backend/src/lib.rs` | Tauriアプリを構成し、公開コマンドを登録する |

文書処理をTauriの起動処理やUIから分離し、単体テスト可能な関数として保つ。差し替える実装が複数になるまでは、トレイトなどの抽象化を追加しない。

## 実装した処理フロー

1. Svelteコンポーネントのマウント後に `get_document` を呼び出す。
2. TauriがRustの `commands::get_document` へ要求を渡す。
3. RustがMarkdown Reader Format v1を検証し、タイトルとテーマを取り出して本文を安全なHTMLへ変換する。
4. Tauriが `title`、`theme`、`html` をフロントエンドへ返す。
5. Svelteが文書テーマと表示モードに対応するCSS変数を使って本文を表示する。

HTTPサーバーや外部公開APIは使用しない。フロントエンドとバックエンドの境界はTauri IPCに限定する。

## セキュリティ方針

- Markdown中の `&`、`<`、`>`、引用符をHTMLエンティティへ変換する。
- UIが `{@html}` で描画する値は、Rust側で生成したHTMLだけに限定する。
- Front Matterのテーマ値は `technical` と `editorial` の許可リストから選び、任意のCSSクラスとして扱わない。
- リンクは相対URL、文書内フラグメント、`http`、`https`、`mailto`だけを出力する。
- v1では画像を出力せず、代替テキストだけを残す。
- Content Security Policyは同梱リソースを基本とし、外部スクリプトを許可しない。
- 将来ファイル選択を追加する場合は、Tauri capabilityと許可するパス範囲を別途設計する。

## 検証方針

| 対象 | 検証 |
| --- | --- |
| Markdown処理 | v1メタデータ、本文構文、注意ブロック、危険なHTMLとURLの扱いをRust単体テストで確認する |
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
| Markdown変換をRust側に置く | 実装済み | 入力の安全化を一箇所に集約できる | MarkdownパーサーとフロントエンドのHTML契約を維持する必要がある |
| Markdown Reader Format v1を定義する | 実装済み | 文書の互換性とエラー条件を明示できる | GFMに対するアプリケーション固有の差分を維持する必要がある |
| Markdownと装飾の責務を分離する | 実装済み | 文書を変えずに表示テーマを変更できる | 独自のテーマ値と注意ブロックは他のMarkdown表示環境では解釈されない |
| CSSをScoped UI、Custom Properties、Markdown専用CSSに分ける | 実装済み | 画面UI、デザイントークン、生成HTMLの装飾境界が明確になる | CSSファイル間のトークン契約を維持する必要がある |
| Front Matterを必須とし、文書テーマを選ぶ | 実装済み | 文書メタデータの所在を統一し、文書単位で意味のある表示系統を指定できる | Front Matterと`theme` はMarkdown標準ではなく本アプリ固有の規約となる |
| 単一文書から始める | 実装済み | 最小のユースケースで全体の接続を検証できる | 文書選択やライブラリ管理は未実装となる |

## 要確認事項

PoC以降は、次を決める必要がある。

1. 利用者が任意のローカルファイルを選択できるようにするか。
2. リンクをアプリ内で遷移させるか、既定ブラウザーで開くか。
3. 将来の形式バージョンで画像表示を追加するか。
4. 文書の編集・保存・検索を対象にするか。
5. 1文書あたりの想定サイズと応答性能をどう定めるか。
