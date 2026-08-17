# CI の構成

ワークフローは 2 本に分かれています。

## `ci.yml` — 静的チェックとテスト

push と pull request のたびに走ります。数分で終わる軽い検査だけを置いています。

| ジョブ | コマンド |
| --- | --- |
| `fmt` | `cargo fmt --all -- --check` |
| `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `check` | `cargo check --workspace --all-targets --all-features` |
| `test` | `cargo nextest run --workspace --all-features` と `cargo test --doc --workspace` |
| `machete` | `cargo machete` |
| `typos` | `typos` |
| `book` | `mdbook build book` と include 記法の展開確認 |

ジョブを分けているのは、`fmt` が落ちても `clippy` の結果が見えるようにするためです。

`book` ジョブがビルドの後に `grep` を挟んでいるのは、
各ライブラリのページが `lib.rs` を include 記法で埋め込んでいるためです。
指し先のパスが壊れると、mdBook は警告を出すだけでビルドには成功し、
展開されなかった記法が出力の HTML にそのまま残ります。
`book/book` にその残骸が現れたら失敗させているので、
crate のファイルを移動・改名してページを直し忘れると、ここで止まります。

`typos` はソースとドキュメントの綴りを検査します。設定はリポジトリルートの `_typos.toml` にあり、
`b"ba"` のようなバイト文字列リテラルは入力データなので検査から外しています。
誤検出が出たら `[default.extend-words]` に語を足すか、`extend-ignore-re` を広げてください。

## `verify.yml` — ジャッジによる検証と Pages のデプロイ

`main` への push と手動実行 (`workflow_dispatch`) で走ります。

1. **setup** — `cargo check` を通し、`oj-resolve` で検証対象を洗い出す
2. **verify** — 検証対象を `SPLIT_SIZE` (現在 4) で分割し、matrix で並列に実行する
3. **docs-and-check** — 結果から Jekyll サイトを生成し、mdBook と束ねて Pages 用アーティファクトにする
4. **deploy** — GitHub Pages に公開する

verify ジョブの結果は `actions/cache` に保存され、次回は `--prev-result` として渡されます。
ライブラリのソースが変わっていない問題は再検証がスキップされます。
キャッシュを無視して全件検証したい場合は、手動実行時に `ignore_prev_result` を有効にしてください。

### 依存解決の粒度

`.competitive-verifier/config.toml` の `list_dependencies_backend` は `kind = "none"` にしています。
この設定では、verify バイナリの依存として `verify/Cargo.toml` に書かれた crate が
実際に使っているかどうかに関わらずすべて記録されます。

```console
$ competitive-verifier oj-resolve --include crates verify \
    --exclude 'crates/*/src/tests.rs' --config .competitive-verifier/config.toml
```

たとえば `verify/Cargo.toml` に `rle` と `fenwick` の 2 つを書いていると、
`rle` しか使っていないバイナリでも次のように解決されます。

```text
verify/src/bin/yukicoder-1469.rs
    deps: crates/fenwick/src/lib.rs, crates/rle/src/lib.rs
```

つまり `fenwick` を書き換えると、`fenwick` を使っていない問題まで再検証されます。
問題数が増えて差分検証が効かないことが実際に負担になったら、
`kind = "cargo-udeps"` に切り替えてください。
未使用依存が除かれる代わりに、setup ジョブに nightly ツールチェインと
`cargo-udeps` のインストールが必要になります。

### 検証対象から外すファイル

`oj-resolve` と `docs` の `exclude` に `crates/*/src/tests.rs` を渡しています。
渡さないと、ユニットテストのファイルもライブラリの 1 つとして解決され、
検証状況ページに検証されないファイルが crate ごとに並びます。
`include` と `exclude` は setup と docs-and-check の 2 か所にあるので、
片方だけ書き換えるとページと検証結果がずれます。

### 必要なシークレット

| 名前 | 用途 |
| --- | --- |
| `YUKICODER_TOKEN` | yukicoder のテストケース取得。[マイページ](https://yukicoder.me/my/page) の API キー |

verify ジョブの Verify ステップに環境変数として渡しています。
未設定のまま yukicoder の問題を検証しようとすると、
テストケースのダウンロードに失敗して verify が落ちます。
Pages のデプロイに使う `GITHUB_TOKEN` は自動で渡されるため、登録は不要です。

## 公開されるページ

Pages のアーティファクトは 2 つのサイトを束ねたものです。

| パス | 内容 | 生成元 |
| --- | --- | --- |
| `/` | verify の実行状況 | competitive-verifier (Jekyll) |
| `/book/` | このドキュメント | mdBook |

Pages にデプロイできるアーティファクトは 1 リポジトリにつき 1 つなので、
両者を同じジョブでビルドして 1 つのディレクトリにまとめています。
