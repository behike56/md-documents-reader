# md-documents-reader

Tauri 2とSvelte 5でMarkdown文書を読むデスクトップアプリのPoCです。`backend/content/` のMarkdownから階層構造JSONを生成して一覧ページに表示します。個別文書はRustバックエンドが安全なHTMLへ変換し、SvelteがTauriコマンド経由で取得します。

## ディレクトリ構成

```text
backend/                 # Tauri/Rustバックエンド
  content/               # 一覧・表示の対象となるファイル
  src/content.rs         # ファイル構造の取得とMarkdownファイルの読み込み
  src/bin/generate_content_index.rs # 一覧JSONの生成コマンド
  src/commands.rs        # フロントエンドへ公開するTauriコマンド
  src/document.rs        # Front Matter検証とMarkdown変換
  tauri.conf.json        # デスクトップアプリ設定
frontend/                # Svelte/Viteフロントエンド
  src/App.svelte         # ファイル構造・文書ページと表示モード切替
  src/ContentTree.svelte # ディレクトリ階層の表示
  src/generated/content-index.json # Markdownから生成した一覧データ
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

`frontend` の開発起動・ビルド時には一覧JSONを自動生成します。手動で更新する場合は `npm --prefix frontend run generate:index` を実行します。

## PoCの範囲

- `backend/content/` のMarkdownファイルから階層構造JSONを生成し、大・中・小カテゴリとページ番号を一覧表示する
- 一覧のリンクから個別の文書ページへ移動する
- 文書ページからファイル構造ページへ戻る
- 一覧に載せる文書は[Markdown Reader Format v2](./docs/design/markdown-format-v2.md)に従う。本文の規則は[v1](./docs/design/markdown-format-v1.md)を引き継ぐ
- YAML Front Matterの`format_version`と`title`を必須とし、`theme: technical / editorial`を解釈する
- 本文はCommonMarkを基礎とし、表、タスクリスト、取り消し線、自動リンクと`info` / `warning` / `success`の注意ブロックを扱う
- Front Matterの`title`をH1として表示し、本文はH2から開始する
- ライト／ダーク表示を切り替えられる
- Markdown内のraw HTMLはRust側で文字列としてエスケープする
- 画像は表示せず、許可していないURLスキームはリンクにしない
- 編集、保存、検索は対象外とする
