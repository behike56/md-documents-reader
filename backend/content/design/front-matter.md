---
format_version: 2
title: Front Matterの書き方
theme: technical
categories:
  large: 設計
  medium: 文書形式
  small: Front Matter
page: 1
tags:
  - YAML
  - メタデータ
description: タイトル、カテゴリー、ページ番号を指定する例。
---

## 基本形

文書の先頭にYAML Front Matterを置きます。`title`は画面の見出しに使われます。

```yaml
---
format_version: 2
title: 文書のタイトル
categories:
  large: 設計
  medium: 文書形式
  small: Front Matter
page: 2
---
```

`categories`の3つの値と`page`は、ファイル構造JSONの生成にも使われます。
