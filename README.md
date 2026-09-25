# md-documents-reader

Markdown 文書をブラウザで読むための PoC です。Rust のHTTPサーバーがテスト用MarkdownをHTMLへ変換し、フロントエンドがAPI経由で取得して表示します。

## 起動

リポジトリルートで次を実行します。

```bash
make run
```

起動後、[http://127.0.0.1:3000](http://127.0.0.1:3000) をブラウザで開きます。

ポートを変える場合は `MD_READER_ADDR` を指定します。

```bash
MD_READER_ADDR=127.0.0.1:8080 make run
```

## PoC の範囲

- `md-documents-reader/content/sample.md` の1文書を表示する
- 対応するMarkdownは見出し、段落、箇条書き、コードブロックに限定する
- Markdown内のHTMLは文字列としてエスケープする
- 文書の選択、編集、保存、検索は対象外とする
