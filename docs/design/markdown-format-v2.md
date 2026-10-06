# Markdown Reader Format v2

## ステータス

採用済み、実装済み。

## 対象

`backend/content/` に置き、ファイル構造ページへ掲載するMarkdown文書に適用する。本文の構文と安全化規則は[Format v1](./markdown-format-v1.md)を引き継ぐ。v1文書の表示処理は残すが、ファイル構造JSONの生成にはv2文書を要求する。

## Front Matter

```yaml
---
format_version: 2
title: Markdownテーマ設計
theme: technical
categories:
  large: 設計
  medium: 表示
  small: Markdown
page: 1
tags:
  - テーマ
description: Markdownの内容と表示テーマの分離を説明する文書。
---
```

| キー | 必須 | 型と制約 |
| --- | --- | --- |
| `format_version` | 必須 | 整数 `2` |
| `title` | 必須 | 前後の空白を除いて空でない文字列 |
| `theme` | 任意 | `technical` または `editorial`。省略時は `technical` |
| `categories` | 必須 | `large`（大）・`medium`（中）・`small`（小）を持つオブジェクト。各値は空でない文字列 |
| `page` | 必須 | 1以上の整数。同じ大・中・小カテゴリの組み合わせ内で重複しない |
| `tags` | 任意 | 空でない文字列の配列 |
| `description` | 任意 | 空でない文字列 |

未知のキー、重複キー、型違い、空のカテゴリー・タグ・説明、`page: 0`はエラーとする。カテゴリーは1文書につき大・中・小の1組とする。別の小カテゴリでは同じページ番号を使用できる。ディレクトリ階層は実際のファイル配置から作る。

## 一覧JSON

`frontend` の開発起動とビルド前に、Rustの `generate_content_index` が `backend/content/` を再帰走査し、`frontend/src/generated/content-index.json` を生成する。JSONの `schema_version` は2。`entries` はディレクトリ階層を持ち、Markdown項目には相対パス、ファイル名、タイトル、大・中・小カテゴリ、ページ番号、タグ、説明を記録する。Markdown以外のファイルとMarkdownを含まないディレクトリは載せない。シンボリックリンクはたどらない。同一のカテゴリ組でページ番号が重複すると生成を失敗させる。

一覧ページは生成済みJSONを読み、文書リンクを表示する。個別文書の本文はTauriコマンドで `backend/content/` から読み込み、RustでHTMLへ変換する。Markdownの追加・変更後は `npm --prefix frontend run generate:index`、または開発起動・ビルドを実行してJSONを更新する。
