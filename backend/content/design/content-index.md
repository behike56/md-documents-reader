---
format_version: 2
title: ファイル構造JSON
theme: technical
categories:
  large: 設計
  medium: 一覧
  small: JSON生成
page: 1
tags:
  - JSON
  - 生成
description: Markdownから一覧データを生成する流れ。
---

## 生成の流れ

開発起動またはフロントエンドのビルド前に、`backend/content/`のMarkdownを再帰的に調べます。各文書のFront Matterを検証し、ファイル構造とメタデータをJSONへ出力します。

:::warning
同じ大・中・小カテゴリの中で`page`が重複すると、JSONの生成は失敗します。
:::
