# Markdown Reader Format v1

## ステータス

採用済み、実装済み。

## 目的

`md-documents-reader`が読み込むMarkdownファイルの構造、メタデータ、本文構文、エラー条件を定める。一般的なMarkdownとの互換性を保ちながら、文書タイトル、表示テーマ、注意ブロックをアプリケーションの規約として追加する。

この仕様はアプリケーションが読み込む文書に適用する。`README.md`や`docs/`以下の設計文書には適用しない。

## ファイル条件

- 拡張子は`.md`とする。
- 文字コードはUTF-8とする。
- 改行コードはLFを推奨し、LFとCRLFを受け付ける。
- ファイルの先頭はFront Matterの開始行`---`とする。先頭の空行やBOMは許可しない。

## 全体構造

文書は、YAML Front MatterとMarkdown本文で構成する。

````markdown
---
format_version: 1
title: "Markdownテーマ設計"
theme: technical
---

## 概要

ここに本文を書く。

```rust
fn main() {
    println!("Hello, Markdown!");
}
```

:::warning
この操作は元に戻せません。
:::
````

## Front Matter

Front Matterは正式なYAMLとして解釈する。開始行と終了行には、それぞれ`---`だけを記述する。

| キー | 必須 | 型 | 制約 |
| --- | --- | --- | --- |
| `format_version` | 必須 | 整数 | `1`だけを受け付ける |
| `title` | 必須 | 文字列 | 前後の空白を除いた結果が空であってはならない |
| `theme` | 任意 | 文字列 | `technical`または`editorial`。省略時は`technical` |

キーの重複、未知のキー、型の不一致、不正な`theme`はエラーとする。仕様を拡張する場合は、既存キーを曖昧に解釈せず`format_version`の更新を検討する。

## タイトルと見出し

- `title`を文書タイトルの唯一の情報源とする。
- アプリケーションは`title`をHTMLの`h1`として本文の先頭に出力する。
- Markdown本文ではH1を使用しない。`# 見出し`を検出した場合はエラーとする。
- 本文の章はH2から開始し、H2からH6までを使用できる。

## 本文構文

本文は[CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/)を基礎とし、次のGitHub Flavored Markdown拡張を有効にする。

- 表
- タスクリスト
- 取り消し線
- 拡張自動リンク

GitHub Flavored Markdownの仕様は[GFM Spec](https://github.github.com/gfm/)を参照する。ただし、安全性と文書構造のため、次の上書き規則を適用する。

| 構文 | 扱い |
| --- | --- |
| H1 | 使用禁止。Front Matterの`title`から生成する |
| raw HTML | 実行せず、文字列としてエスケープする |
| リンク | 相対URL、文書内フラグメント、`http`、`https`、`mailto`だけをリンクとして出力する |
| 許可されていないURLスキーム | リンクを解除し、表示文字列だけを残す |
| 画像 | v1では画像を出力せず、代替テキストだけを残す |

このため、本仕様はCommonMarkとGFMを基礎にしたアプリケーション固有プロファイルであり、GFM完全互換ではない。

## 注意ブロック

アプリケーション固有の注意ブロックとして、`info`、`warning`、`success`を使用できる。

```markdown
:::info
補足情報を書く。
:::

:::warning
注意事項を書く。
:::

:::success
成功条件を書く。
:::
```

注意ブロックには次の制約を適用する。

- 開始行と終了行には、マーカーだけを記述する。
- 開始マーカーと終了マーカーを必ず対応させる。
- 注意ブロックを入れ子にしない。
- ブロック内部では通常のMarkdown本文構文を使用できる。
- fenced code block内の同じ文字列は注意ブロックとして解釈しない。

## エラー条件

次の場合、文書全体の読み込みを失敗させ、画面にエラーを表示する。

- Front Matterが存在しない、または閉じられていない。
- Front MatterがYAMLとして不正である。
- 必須キーがない、未知のキーがある、または値の型が異なる。
- `format_version`が`1`ではない。
- `title`が空である。
- 本文にH1がある。
- 注意ブロックが閉じられていない、入れ子になっている、または開始せずに閉じられている。

## 実装との対応

| 責務 | 実装 |
| --- | --- |
| Front Matterの切り出しと必須判定 | `backend/src/document.rs` |
| YAMLの型検証 | `yaml_serde`と`DocumentMetadata` |
| CommonMarkおよびGFM変換 | `pulldown-cmark` |
| H1、HTML、リンク、画像、注意ブロックの規約 | Markdownイベント変換処理 |
| 利用者向けエラー表示 | `backend/src/commands.rs`と`frontend/src/App.svelte` |

## 検証方針

単体テストでは、正常なv1文書に加えて、Front Matterの欠落、不正YAML、必須キー欠落、未知キー、不正テーマ、未対応バージョン、空タイトル、H1、raw HTML、危険なリンク、画像、注意ブロックの境界を確認する。同梱サンプルもv1文書として読み込めることを確認する。
