# はじめに

競技プログラミング用の Rust ライブラリです。アルゴリズムごとに crate を分け、
[Library Checker](https://judge.yosupo.jp/) や [yukicoder](https://yukicoder.me/) の問題を使って
CI で正当性を検証しています。

- リポジトリ: <https://github.com/topi-banana/library>
- verify の実行状況: <https://topi-banana.github.io/library/>
- この解説ページ: <https://topi-banana.github.io/library/book/>

## 構成

```text
library/
├── crates/          各アルゴリズムの crate
│   ├── div_conquer/
│   ├── fibonacci/
│   ├── kmp/
│   ├── mo/
│   ├── permutation/
│   ├── ranges/
│   ├── rle/
│   └── vec2d/
├── verify/          ジャッジの 1 問 = 1 バイナリ
│   └── src/bin/
└── book/            このドキュメント (mdBook)
```

ワークスペースのメンバーは `crates/*` と `verify` です。
各 crate は `publish = false` で、crates.io には公開しません。

## 使い方

`Cargo.toml` にパス依存として書きます。

```toml
[dependencies]
rle = { git = "https://github.com/topi-banana/library" }
```

コンテスト本番では単一ファイルにまとめる必要があるため、
[`cargo-equip`](https://github.com/qryxip/cargo-equip) などのバンドラを併用してください。

## ライセンス

[CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/deed.ja) です。
出典表記なしで自由に利用できます。
