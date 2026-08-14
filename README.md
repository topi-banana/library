# library

[![verify](https://github.com/topi-banana/library/actions/workflows/verify.yml/badge.svg)](https://github.com/topi-banana/library/actions/workflows/verify.yml)
[![CI](https://github.com/topi-banana/library/actions/workflows/ci.yml/badge.svg)](https://github.com/topi-banana/library/actions/workflows/ci.yml)
[![GitHub Pages](https://img.shields.io/static/v1?label=GitHub+Pages&message=+&color=brightgreen&logo=github)](https://topi-banana.github.io/library/)

競技プログラミング用の Rust ライブラリ。
アルゴリズムごとに crate を分け、[Library Checker](https://judge.yosupo.jp/) や
[yukicoder](https://yukicoder.me/) の問題で正当性を検証しています。

- **検証状況**: <https://topi-banana.github.io/library/>
- **解説ドキュメント**: <https://topi-banana.github.io/library/book/>

## 構成

```text
library/
├── crates/                 各アルゴリズムの crate
│   ├── div_conquer/        平方分割 (ブロック分割による区間クエリ)
│   ├── fibonacci/          フィボナッチ数列 (行列累乗と列挙)
│   ├── kmp/                Knuth-Morris-Pratt 法 (列の検索)
│   ├── mo/                 Mo's algorithm (区間クエリのオフライン処理)
│   ├── permutation/        順列 (辞書順の遷移と全列挙)
│   ├── ranges/             半開区間の集合 (統合・削除・被覆判定)
│   └── rle/                連長圧縮 (Run-Length Encoding)
├── verify/src/bin/         ジャッジの 1 問 = 1 バイナリ
├── book/                   解説ドキュメント (mdBook)
├── .competitive-verifier/  検証ツールの設定と検証状況ページの素材
└── .github/workflows/      ci.yml (静的チェック) と verify.yml (検証 + Pages)
```

## 開発

```console
$ cargo fmt --all
$ cargo clippy --workspace --all-targets --all-features -- -D warnings
$ cargo check --workspace --all-targets --all-features
$ cargo nextest run --workspace --all-features
$ cargo test --doc --workspace --all-features
$ cargo machete
$ mdbook build book
```

ライブラリの追加手順と verify の書き方は
[解説ドキュメントの開発セクション](https://topi-banana.github.io/library/book/dev/add-library.html)
にまとめています。

### 必要なツール

| ツール | インストール |
| --- | --- |
| cargo-nextest | `cargo install cargo-nextest --locked` |
| cargo-machete | `cargo install cargo-machete --locked` |
| mdBook | `cargo install mdbook --locked` |
| competitive-verifier | `pip install competitive-verifier` |

## ライセンス

[CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/deed.ja)
