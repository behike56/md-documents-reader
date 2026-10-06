---
format_version: 2
title: "Markdownテーマ設計"
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

## 概要

これは、バックエンドから読み込んだテスト用の Markdown 文書です。

## 確認できること

- Rust のバックエンドが文書を提供する
- Svelte のフロントエンドが Tauri コマンドを呼び出す
- 見出し、段落、箇条書き、コードブロックを表示する
- Front Matter で文書テーマを選択する

:::info
Markdownは内容と意味を持ち、配色や背景はテーマ側で管理します。
:::

:::warning
Markdown内のraw HTMLは実行せず、文字列として表示します。
:::

```rust
fn main() {
    println!("Hello, Markdown!");
}
```
