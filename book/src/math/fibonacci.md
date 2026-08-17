# Fibonacci — フィボナッチ数列

`F(0) = 0`、`F(1) = 1`、`F(n) = F(n - 1) + F(n - 2)` で定まる数列を扱います。
`fibonacci_matrix_pow` の値の型は `u128` 固定で、`Fibonacci` は
`AddAssign + Clone` を満たす任意の型を載せられます。

- 実装: [`crates/fibonacci/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/fibonacci/src/lib.rs) — [全文はこのページの末尾](#ソース)
- verify:
  - `fibonacci_matrix_pow` — [yukicoder No.786 京都大学の過去問](https://yukicoder.me/problems/no/786)
  - `Fibonacci` — [yukicoder No.195 フィボナッチ数列の理解(2)](https://yukicoder.me/problems/no/195)

第 `n` 項だけが欲しい [`fibonacci_matrix_pow`](#fibonacci_matrix_pow) と、
先頭から順に列挙する [`Fibonacci`](#fibonacci) の 2 つがあります。
第 `n` 項までまとめて使うなら、1 項あたり `O(1)` で進むイテレータのほうが速く、
飛び飛びの 1 項だけが欲しいなら行列累乗のほうが速い、という使い分けになります。

## API

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `fibonacci_matrix_pow(n)` | `O(log n)` (2x2 の行列積を高々 `2 log n` 回) | 第 `n` 項 `F(n)` を `u128` で返す。`F(0) = 0` |
| `Fibonacci(a, b)` | `O(1)` | 直前の 2 項を持つ無限イテレータを作る。フィールドは公開されている |
| `Fibonacci::next()` | 要素型の加算 1 回とクローン 2 回 (`u128` なら `O(1)`) | 次の項を返す。`None` にはならない |

添字は `F(0) = 0`、`F(1) = F(2) = 1` です。
`fibonacci_matrix_pow(0)` は `0` を返しますが、イテレータが最初に返すのは `F(1) = 1` で、
`F(0)` は出てこないことに注意してください。

## fibonacci_matrix_pow

```rust
use fibonacci::fibonacci_matrix_pow;

assert_eq!(fibonacci_matrix_pow(0), 0);
assert_eq!(fibonacci_matrix_pow(1), 1);
assert_eq!(fibonacci_matrix_pow(2), 1);
assert_eq!(fibonacci_matrix_pow(50), 12586269025);
```

漸化式を行列で書くと

```text
| F(n + 1) |   | 1  1 | | F(n)     |
|          | = |      | |          |
| F(n)     |   | 1  0 | | F(n - 1) |
```

なので、`[[1, 1], [1, 0]]` の `n - 1` 乗の左上成分が `F(n)` になります。
これを繰り返し二乗法で求めるため、行列積の回数が `O(log n)` になります。

`n = 0` だけは `n - 1` が引けないので、行列を作る前に `0` を返して分岐しています。

## Fibonacci

直前の 2 項を持つだけのタプル構造体 `Fibonacci<T>(T, T)` です。
`next()` は `(.0, .1)` を `(.1, .0 + .1)` に更新して新しい `.0` を返します。

要素型 `T` は `AddAssign + Clone` さえ満たせばよいので、必要な項数に合わせて選べます。
ただしリテラルだけ書くと `i32` に推論され、[オーバーフロー](#オーバーフロー)が
`F(46)` を返す呼び出しで起きてしまいます。以下の例のように型は明示してください。

```rust
use fibonacci::Fibonacci;

let fib = Fibonacci(0u128, 1u128);
assert_eq!(fib.take(10).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);

// 必要な項数が少なければ狭い型でよい
let fib = Fibonacci(0u64, 1u64);
assert_eq!(fib.take(5).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5]);
```

無限イテレータなので、`take` や `take_while` で必ず打ち切ってください。

```rust
use fibonacci::Fibonacci;

// 100 以下の項
let fib = Fibonacci(0u128, 1u128);
let small: Vec<_> = fib.take_while(|&f| f <= 100).collect();
assert_eq!(small, vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);

// 10^9 を最初に超える項が何項目か
let fib = Fibonacci(0u128, 1u128);
let (i, f) = fib
    .enumerate()
    .find(|&(_, f)| f > 1_000_000_000)
    .map(|(i, f)| (i + 1, f))
    .unwrap();
assert_eq!((i, f), (45, 1134903170));
```

フィールドは公開されているので、途中の項から再開できます。
直前に返した項を `F(k)` として `.0 == F(k)`、`.1 == F(k + 1)` が不変条件です。
`Fibonacci(0, 1)` は `k = 0` にあたります。

```rust
use fibonacci::Fibonacci;

// F(10) = 55、F(11) = 89 から再開する
let fib = Fibonacci(55u128, 89u128);
assert_eq!(fib.take(3).collect::<Vec<_>>(), vec![89, 144, 233]);
```

初項を変えれば、同じ漸化式の別の数列 (リュカ数など) にも使えます。

```rust
use fibonacci::Fibonacci;

// リュカ数 L(1) = 1, L(2) = 3, ...
let lucas = Fibonacci(2u128, 1u128);
assert_eq!(lucas.take(6).collect::<Vec<_>>(), vec![1, 3, 4, 7, 11, 18]);
```

## verify

yukicoder 2 問で検証しています。

### yukicoder No.786 京都大学の過去問

[No.786 京都大学の過去問](https://yukicoder.me/problems/no/786) は、
`N` 段の階段を 1 歩 1 段または 2 段で昇る方法が何通りあるかを答える問題です。

`n` 段目に来る直前は `n - 1` 段目か `n - 2` 段目なので、
通り数を `f(n)` とすると `f(n) = f(n - 1) + f(n - 2)` になります。
`f(1) = 1`、`f(2) = 2` なので `f(n) = F(n + 1)` です。

```rust
use fibonacci::fibonacci_matrix_pow;

assert_eq!(fibonacci_matrix_pow(3 + 1), 3);
assert_eq!(fibonacci_matrix_pow(9 + 1), 55);
```

欲しいのが 1 項だけなので、列挙せずに行列累乗で求めています。
`N <= 50` で `F(51) = 20365011074` なので、32 bit 整数には収まりません。

実際の verify コードは
[`verify/src/bin/yukicoder-786.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-786.rs)
です。

### yukicoder No.195 フィボナッチ数列の理解(2)

[No.195 フィボナッチ数列の理解(2)](https://yukicoder.me/problems/no/195) は、
`F(1) = A`、`F(2) = B` から同じ漸化式で伸ばした「`(A, B)` フィボナッチ数列」が
与えられた `X, Y, Z` をすべて含むような正整数の対 `(A, B)` のうち、
辞書順最小のものを答える問題です。

初項を変えられる `Fibonacci` がそのまま使えます。
ただし最初に返るのは `b` (第 2 項) なので、`a` を頭に付けて添字を揃えます。

```rust
use fibonacci::Fibonacci;

// (1, 3) フィボナッチ数列の第 1 項から
let (a, b) = (1u64, 3u64);
let terms: Vec<_> = std::iter::once(a).chain(Fibonacci(a, b)).take(6).collect();
assert_eq!(terms, vec![1, 3, 4, 7, 11, 18]);
```

第 `k` 項は `F_{A,B}(k) = F(k - 2) · A + F(k - 1) · B` と書けます
(`F(-1) = 1`、`F(0) = 0` と読みます)。
`A, B >= 1` より `F_{A,B}(k) >= F(k)` で、[`Fibonacci` の例](#fibonacci)で見たとおり
`F(45) = 1134903170 > 10^9` なので、`10^9` 以下の値が入るのは第 44 項までです。

`X, Y, Z` から重複を除いて 2 値以上が残るなら、その中の相異なる 2 値を取ります。
同じ値が数列に 2 回現れるのは `A = B` のときの第 1, 2 項だけ
(第 2 項以降は `A >= 1` より狭義単調増加) なので、
相異なる 2 値は必ず別の添字 `i != j` に入ります。そこで `i`、`j` を決め打ちすると、

```text
F(i - 2)·A + F(i - 1)·B = (最小の値)
F(j - 2)·A + F(j - 1)·B = (次に小さい値)
```

という連立方程式になります。行列式は `±F(|i - j|)` で `i != j` なら `0` にならないため、
`(A, B)` が一意に定まります。答えの `(A, B)` は必ずどれかの `(i, j)` に対応するので、
`44 * 43` 通りの `(i, j)` をすべて試し、得られた `(A, B)` で実際に数列を並べて
残りの値も含むかを確かめれば、条件を満たすもの全体が漏れなく列挙できます。

`X = Y = Z` のときだけは 2 値が取れませんが、`(A, B) = (1, X)` が必ず `X` を含むので
`A = 1` で確定し、`X = F(k - 2) + F(k - 1)·B` を満たす最小の `B` を探すだけになります。

実際の verify コードは
[`verify/src/bin/yukicoder-195.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-195.rs)
です。

## オーバーフロー

`u128` に収まる最大の項は `F(186)` (約 `3.3 * 10^38`) です。
これを超えたときの挙動はビルドのオーバーフロー検査に従うため、
debug ビルドでは panic、release ビルドでは `2^128` を法とした値になります。

| | debug (検査あり) | release (検査なし) |
| --- | --- | --- |
| `fibonacci_matrix_pow(n)` | `n <= 186`。`n >= 187` で panic | `2^128` を法とした値 |
| `Fibonacci<u128>` | `F(185)` まで。`F(186)` の生成で panic | `F(186)` まで正しい |

イテレータだけ 1 項手前で止まるのは、`next()` が返す項の 1 つ先まで計算するためです。
`F(186)` を返す呼び出しは同時に `F(187)` を作ってしまい、そこであふれます。

`Fibonacci` の上限は載せた型に従うので、狭い型を選べばその分早く尽きます。
「型に収まる最大の項の 1 つ手前まで返せる」という関係はどの型でも同じです。

| 要素型 | 収まる最大の項 | debug で返せるところまで |
| --- | --- | --- |
| `i32` | `F(46) = 1836311903` | `F(45)` まで。`F(46)` を返す呼び出しで panic |
| `u64` | `F(93)` (約 `1.2 * 10^19`) | `F(92)` まで。`F(93)` を返す呼び出しで panic |
| `u128` | `F(186)` (約 `3.3 * 10^38`) | `F(185)` まで。`F(186)` を返す呼び出しで panic |

行列累乗のほうは、途中の値が `F(n)` より大きくならないように書いてあります。
繰り返し二乗法のループで最後にもう一度二乗すると、`n - 1 = 128` のときに
`M^128` から `M^256` を作ろうとしてあふれるため、
結果に使わないと分かっている最後の二乗は飛ばしています。

より大きな項が必要なら、`Fibonacci` は要素型を多倍長整数や剰余環の型に差し替えられます
(`AddAssign + Clone` があれば載ります)。`fibonacci_matrix_pow` は `u128` 固定なので、
同じことをするには行列積のほうを書き換える必要があります。
競技プログラミングでは答えを `10^9 + 7` などで割った余りにする問題が多く、
その場合は行列積の各要素を法の下で計算することになります。

## ソース

`crates/fibonacci/src/lib.rs` の全文です。コードブロック右上のボタンでまるごとコピーできます。
リポジトリのファイルをそのまま埋め込んでいるので、この表示が実装とずれることはありません。

末尾の `#[cfg(test)] mod tests;` はユニットテストを読み込む 2 行です。
提出先ではテストがコンパイルされないため、貼り付けたままで構いません。

```rust,ignore
{{#include ../../../crates/fibonacci/src/lib.rs}}
```
