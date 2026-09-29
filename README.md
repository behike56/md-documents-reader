# md-documents-reader

Tauri 2とSvelte 5でMarkdown文書を読むデスクトップアプリのPoCです。Rustバックエンドが同梱MarkdownをHTMLへ変換し、SvelteフロントエンドがTauriコマンド経由で取得して表示します。

## ディレクトリ構成

```text
backend/                 # Tauri/Rustバックエンド
  content/sample.md      # 表示するテスト文書
  src/commands.rs        # フロントエンドへ公開するTauriコマンド
  src/document.rs        # Markdown変換
  tauri.conf.json        # デスクトップアプリ設定
frontend/                # Svelte/Viteフロントエンド
  src/App.svelte         # 文書表示画面
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
- 対応するMarkdownは見出し、段落、箇条書き、コードブロックに限定する
- Markdown内のHTMLはRust側で文字列としてエスケープする
- 文書の選択、編集、保存、検索は対象外とする
