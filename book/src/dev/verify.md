# verify を書く

正当性の検証には [competitive-verifier](https://github.com/competitive-verifier/competitive-verifier) を使います。
`verify` crate のバイナリ 1 つが、ジャッジの 1 問に対応します。

対応するジャッジ問題が無いライブラリは、ユニットテストと doctest だけで検証します。

## ファイルを置く

`verify/src/bin/` に `<ジャッジ名>-<問題名>.rs` を作り、
先頭に `PROBLEM` を宣言します。

```rust
// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1469

use std::io::{self, Read, Write};

use rle::Rle;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let s = input.trim_end();

    // 問題を解く

    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(s.as_bytes())
}
```

`// competitive-verifier: PROBLEM <URL>` の行がないファイルは検証対象になりません。
Library Checker のほか、AOJ や yukicoder の URL も指定できます。

### yukicoder を使うとき

yukicoder はテストケースの取得に API トークンが必要です。
[yukicoder のマイページ](https://yukicoder.me/my/page) で API キーを発行し、
リポジトリのシークレットに `YUKICODER_TOKEN` という名前で登録してください。
`verify.yml` の Verify ステップから環境変数として渡しています。

```yaml
      - name: Verify
        uses: competitive-verifier/actions/verify@v2
        env:
          YUKICODER_TOKEN: ${{ secrets.YUKICODER_TOKEN }}
```

シークレットが未設定だと、テストケースのダウンロードに失敗して verify が落ちます。
ローカルで動かすときも同じ名前の環境変数を設定してください。

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
$ printf 'programming\n' | ./target/debug/yukicoder-1469
programing
```
