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

ワークスペースの lint 設定では `missing_docs` を `allow` にしていますが、
公開アイテムにはドキュメントコメントを書いてください。
CI は `-D warnings` で clippy を走らせるので、他の警告も残せません。

パニックしうる関数には `# Panics` セクションを、
`Result` を返す関数には `# Errors` セクションを書いてください。

crate レベルのドキュメントには、実行可能なコード例を 1 つ置いておくと
doctest がそのまま回帰テストになります。

## 3. ユニットテストを書く

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
