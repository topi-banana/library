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
