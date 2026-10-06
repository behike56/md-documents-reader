# md-documents-reader

Tauri 2とSvelte 5でMarkdown文書を読むデスクトップアプリのPoCです。Rustバックエンドが同梱Markdownを安全なHTMLへ変換し、SvelteフロントエンドがTauriコマンド経由で取得してテーマ付きで表示します。

## ディレクトリ構成

```text
backend/                 # Tauri/Rustバックエンド
  content/sample.md      # 表示するテスト文書
  src/commands.rs        # フロントエンドへ公開するTauriコマンド
  src/document.rs        # Front Matter検証とMarkdown変換
  tauri.conf.json        # デスクトップアプリ設定
frontend/                # Svelte/Viteフロントエンド
  src/App.svelte         # 文書表示画面と表示モード切替
  src/styles/tokens.css  # 配色などのデザイントークン
  src/styles/markdown.css # Markdown本文専用の表示規則
```

## 前提

- Rust 1.98.1
- Node.js 22.12以降
- OSごとのTauri開発用依存関係

## 起動

初回のみフロントエンド依存関係をインストールします。

```bash
make install
```

その後、Tauriデスクトップアプリを起動します。

```bash
make run
```

## 確認

```bash
make check
make test
```

`make check` はSvelteのプロダクションビルドとRustのformat、Clippy、compile checkを実行します。

## PoCの範囲

- `backend/content/sample.md` の1文書をデスクトップ画面に表示する
- 入力は[Markdown Reader Format v1](./docs/design/markdown-format-v1.md)に従う
- YAML Front Matterの`format_version`と`title`を必須とし、`theme: technical / editorial`を解釈する
- 本文はCommonMarkを基礎とし、表、タスクリスト、取り消し線、自動リンクと`info` / `warning` / `success`の注意ブロックを扱う
- Front Matterの`title`をH1として表示し、本文はH2から開始する
- ライト／ダーク表示を切り替えられる
- Markdown内のraw HTMLはRust側で文字列としてエスケープする
- 画像は表示せず、許可していないURLスキームはリンクにしない
- 文書の選択、編集、保存、検索は対象外とする
