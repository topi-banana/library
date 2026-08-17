# ライブラリを追加する

新しいアルゴリズムは `crates/` 以下に 1 crate として追加します。
ワークスペースの `members` は `crates/*` を glob で拾っているため、
ルートの `Cargo.toml` を書き換える必要はありません。

## 1. crate を作る

```console
$ cargo new --lib crates/fenwick
```

`crates/fenwick/Cargo.toml` をワークスペース継承の形に整えます。

```toml
[package]
name = "fenwick"
version.workspace = true
edition.workspace = true
publish.workspace = true
repository.workspace = true
license.workspace = true

[lints]
workspace = true
```

パッケージ名とライブラリ名は揃えてください。
`[lib] name` だけを変えると `cargo machete` が依存を未使用と誤検知します。

## 2. 実装する

`src/lib.rs` にはコンテストへ提出するコードだけを置き、
ユニットテストは隣の `src/tests.rs` に分けます。

```text
crates/fenwick/
├── Cargo.toml
└── src/
    ├── lib.rs      提出するコード
    └── tests.rs    ユニットテスト
```

`lib.rs` の末尾でテストモジュールを宣言します。

```rust,ignore
#[cfg(test)]
mod tests;
```

この 2 行以外は提出するコードそのものなので、コンテスト中は `lib.rs` を
丸ごとコピーして貼るだけで済みます。`cfg(test)` で無効になったモジュールは
ファイルを探しにいかないため、2 行を残したまま提出してもコンパイルは通ります
(貼り付け先で `cargo test` を走らせるなら消してください)。

ワークスペースの lint 設定では `missing_docs` を `allow` にしていますが、
公開アイテムにはドキュメントコメントを書いてください。
CI は `-D warnings` で clippy を走らせるので、他の警告も残せません。

パニックしうる関数には `# Panics` セクションを、
`Result` を返す関数には `# Errors` セクションを書いてください。

crate レベルのドキュメントには、実行可能なコード例を 1 つ置いておくと
doctest がそのまま回帰テストになります。

## 3. ユニットテストを書く

テストは `src/tests.rs` に書きます。`#[cfg(test)]` は `lib.rs` の宣言側に付いているので、
`tests.rs` の先頭では要りません。モジュールの位置は `lib.rs` の中に
`mod tests { .. }` を直接書いたときと同じなので、`use super::*;` はそのまま使えて、
crate 直下の非公開アイテムにも触れます。

境界条件 (空、要素 1 個、範囲外) と、
素朴な実装との突き合わせを書いておくと、verify が落ちたときの切り分けが速くなります。

```console
$ cargo nextest run --workspace
$ cargo test --doc --workspace
```

`cargo nextest` は doctest を実行しません。ドキュメント内のコード例は
`cargo test --doc` で別に確認します。

## 4. verify を追加する

[verify を書く](./verify.md) を参照してください。
対応するジャッジ問題が無いライブラリは、この手順を飛ばして
ユニットテストと doctest だけで検証します。

## 5. ドキュメントを追加する

`book/src/` に解説を書き、`book/src/SUMMARY.md` に項目を追加します。
`SUMMARY.md` に載っていないページはビルド対象に含まれません。
分類のインデックス (`book/src/seq/index.md` など) の表にも 1 行足してください。

ページの構成は既存のものに揃えます。

| 節 | 内容 |
| --- | --- |
| 冒頭 | 何をするライブラリか、全体の計算量、実装へのリンク、verify した問題 |
| `## API` | 公開している項目を**すべて**表に並べる |
| `## 使用例` | `use` から書いた、そのまま動くコード例 |
| `## 注意点` | 踏みやすい罠。書き忘れると verify で初めて気づく類のもの |
| `## verify` | 問題ごとに、何がどう問われているか |
| `## 実装メモ` | 実装の選択とその理由 |
| `## ソース` | `lib.rs` の全文 |

### API の表

`| 項目 | 計算量 | 説明 |` の 3 列で、公開している関数・メソッド・トレイトの項目を
1 つずつ挙げます。抜けがあると、読む側は `lib.rs` を読み直すことになります。

- **計算量は必ず埋める。** `O(1)` や `O(log n)` のほか、
  「ならし `O(log n)`」「最後まで回すと合計 `O(n)`」のように条件を添えて構いません。
  `n` が何を数えたものかは表の前で定義しておきます。
- **コールバックを受け取るライブラリは、時間ではなく呼び出し回数で書く。**
  1 回の重さは利用者が決めるものなので、
  [`DivConquer`](../query/div_conquer.md#api) や [`Mo`](../query/mo.md#api) では
  「`cacher` を `b` 回」「4 つ合わせて `O(n √q)` 回」のように示しています。
- **型が 2 つ以上あるなら、型ごとに表を分ける。**
  [`Ranges`](../set/ranges.md#api) や [`DivConquer`](../query/div_conquer.md#api) の形です。

### ソースの節

コンテスト中は `lib.rs` を丸ごと貼り付けて使うため、
ページの末尾でその全文を埋め込み、コードブロックのコピーボタンから
1 クリックで取れるようにしています。
GitHub へのリンクだけだと、ページを離れて別のコピー操作をすることになります。

節ごと `book/src/seq/rle.md` の `## ソース` からコピーして、
crate 名の部分だけ書き換えるのが確実です。
ファイルの中身を埋め込んでいるので、実装を変えても放置してよく、
ページとソースがずれることはありません。
埋め込んだブロックの言語指定は `rust,ignore` です。
crate に依存するコードは Playground で動かせないため `book.toml` で `runnable = false` にしてあり、
`ignore` を付けたブロックは `mdbook test` を回したときにも実行対象から外れます
(その `mdbook test` は、現在 CI では走らせていません)。

パスが壊れると mdBook は警告を出すだけでビルドを通してしまうので、
CI では[出力に残骸が無いか](./ci.md)を別途確かめています。
