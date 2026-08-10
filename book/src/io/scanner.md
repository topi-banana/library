# Scanner

空白区切りの入力を先頭から順に読み取ります。

- 実装: [`crates/scanner/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/scanner/src/lib.rs)

入力全体を一度 `String` に読み込んでからトークンに切り出します。
1 行ずつ読む実装と比べてシステムコールの回数が減るため、
入力が数十万行に及ぶ問題でも入力待ちがボトルネックになりにくくなります。

## API

| メソッド | 説明 |
| --- | --- |
| `Scanner::new(reader)` | `Read` を実装した値から作る |
| `Scanner::from_stdin()` | 標準入力から作る |
| `next_token()` | 次のトークンを `&str` で返す。尽きていれば `None` |
| `read::<T>()` | 次のトークンを `T` にパースする |
| `read_vec::<T>(n)` | 次の `n` 個のトークンを `Vec<T>` にする |

`read` と `read_vec` は入力が尽きた場合とパースに失敗した場合にパニックします。
競技プログラミングでは入力形式が保証されているため、`Result` を返さない設計にしています。

## 使用例

```rust
use scanner::Scanner;
use std::io::{self, BufWriter, Write};

let mut sc = Scanner::from_stdin()?;
let stdout = io::stdout();
let mut out = BufWriter::new(stdout.lock());

let n: usize = sc.read();
let a: Vec<i64> = sc.read_vec(n);

writeln!(out, "{}", a.iter().sum::<i64>())?;
out.flush()?;
# Ok::<(), io::Error>(())
```

出力側は `BufWriter` で包んでください。
`println!` を毎行呼ぶと 1 行ごとに標準出力のロックとフラッシュが走り、
出力が多い問題では TLE の原因になります。
