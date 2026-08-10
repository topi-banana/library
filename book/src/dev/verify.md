# verify を書く

正当性の検証には [competitive-verifier](https://github.com/competitive-verifier/competitive-verifier) を使います。
`verify` crate のバイナリ 1 つが、ジャッジの 1 問に対応します。

## ファイルを置く

`verify/src/bin/` に `<ジャッジ名>-<問題名>.rs` を作り、
先頭に `PROBLEM` を宣言します。

```rust
// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/unionfind

use std::io::{self, BufWriter, Write};

use dsu::Dsu;
use scanner::Scanner;

fn main() -> io::Result<()> {
    let mut sc = Scanner::from_stdin()?;
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    // 問題を解く

    out.flush()
}
```

`// competitive-verifier: PROBLEM <URL>` の行がないファイルは検証対象になりません。
Library Checker のほか、AOJ や yukicoder の URL も指定できます。

使う crate は `verify/Cargo.toml` の `[dependencies]` に追加してください。
バイナリごとに使う crate は違いますが、`cargo machete` は crate 全体を見るため、
どれか 1 つのバイナリで使われていれば未使用扱いにはなりません。

## 検証の仕組み

competitive-verifier は `PROBLEM` の URL からテストケースをダウンロードし、
`cargo build --release --bin <名前>` でビルドしたバイナリを全ケースに対して実行します。
出力は問題ごとの checker で判定されるため、
複数解がある問題でも「サンプルと文字列一致するか」ではなく正しく判定されます。

依存ファイルの解決には `cargo metadata` と `cargo check --workspace --all-targets` の
出力を使います。ワークスペース全体がコンパイルできない状態では検証が始まりません。

## ローカルで動かす

```console
$ pip install competitive-verifier
$ competitive-verifier oj-resolve --config config.toml > verify_files.json
$ competitive-verifier verify
```

テストケースのダウンロードには時間がかかります。
特定の問題だけ試したい場合は、まずサンプル入力を直接流し込むのが手軽です。

```console
$ cargo build --bins
$ printf '4 7\n1 0 1\n0 0 1\n0 2 3\n1 0 1\n1 1 2\n0 0 2\n1 1 3\n' \
    | ./target/debug/library-checker-unionfind
```
