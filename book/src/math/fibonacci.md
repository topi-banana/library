# Fibonacci — フィボナッチ数列

`F(0) = 0`、`F(1) = 1`、`F(n) = F(n - 1) + F(n - 2)` で定まる数列を扱います。
値の型はいずれも `u128` 固定です。

- 実装: [`crates/fibonacci/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/fibonacci/src/lib.rs)
- verify: 対応するジャッジ問題が無いため、ユニットテストと doctest だけで検証しています。

第 `n` 項だけが欲しい [`fibonacci_matrix_pow`](#fibonacci_matrix_pow) と、
先頭から順に列挙する [`Fibonacci`](#fibonacci) の 2 つがあります。
第 `n` 項までまとめて使うなら、1 項あたり `O(1)` で進むイテレータのほうが速く、
飛び飛びの 1 項だけが欲しいなら行列累乗のほうが速い、という使い分けになります。

## API

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `fibonacci_matrix_pow(n)` | `O(log n)` | 第 `n` 項 `F(n)` を返す。`F(0) = 0` |
| `Fibonacci { a, b }` | — | `F(1)` から順に返す無限イテレータ |
| `Fibonacci::next()` | `O(1)` | 次の項を返す。`None` にはならない |

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

`a` と `b` に直前の 2 項を持つだけの構造体です。
`next()` は `(a, b)` を `(b, a + b)` に更新して新しい `a` を返します。

```rust
use fibonacci::Fibonacci;

let fib = Fibonacci { a: 0, b: 1 };
assert_eq!(fib.take(10).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
```

無限イテレータなので、`take` や `take_while` で必ず打ち切ってください。

```rust
use fibonacci::Fibonacci;

// 100 以下の項
let fib = Fibonacci { a: 0, b: 1 };
let small: Vec<_> = fib.take_while(|&f| f <= 100).collect();
assert_eq!(small, vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);

// 10^9 を最初に超える項が何項目か
let fib = Fibonacci { a: 0, b: 1 };
let (i, f) = fib
    .enumerate()
    .find(|&(_, f)| f > 1_000_000_000)
    .map(|(i, f)| (i + 1, f))
    .unwrap();
assert_eq!((i, f), (45, 1134903170));
```

フィールドは公開されているので、途中の項から再開できます。
直前に返した項を `F(k)` として `a == F(k)`、`b == F(k + 1)` が不変条件です。
`Fibonacci { a: 0, b: 1 }` は `k = 0` にあたります。

```rust
use fibonacci::Fibonacci;

// F(10) = 55、F(11) = 89 から再開する
let fib = Fibonacci { a: 55, b: 89 };
assert_eq!(fib.take(3).collect::<Vec<_>>(), vec![89, 144, 233]);
```

初項を変えれば、同じ漸化式の別の数列 (リュカ数など) にも使えます。

```rust
use fibonacci::Fibonacci;

// リュカ数 L(1) = 1, L(2) = 3, ...
let lucas = Fibonacci { a: 2, b: 1 };
assert_eq!(lucas.take(6).collect::<Vec<_>>(), vec![1, 3, 4, 7, 11, 18]);
```

## オーバーフロー

`u128` に収まる最大の項は `F(186)` (約 `3.3 * 10^38`) です。
これを超えたときの挙動はビルドのオーバーフロー検査に従うため、
debug ビルドでは panic、release ビルドでは `2^128` を法とした値になります。

| | debug (検査あり) | release (検査なし) |
| --- | --- | --- |
| `fibonacci_matrix_pow(n)` | `n <= 128`。`n >= 129` で panic | `n <= 186` まで正しい |
| `Fibonacci` | `F(185)` まで。`F(186)` の生成で panic | `F(186)` まで正しい |

イテレータが 1 項手前で止まるのは、`next()` が返す項の 1 つ先まで計算するためです。
`F(186)` を返す呼び出しは同時に `F(187)` を作ってしまい、そこであふれます。

`fibonacci_matrix_pow` が `n = 129` で止まるのは、`F(n)` の大きさとは別の理由です。
繰り返し二乗法のループが、もう使わないと分かっている最後の二乗まで実行するため、
`n - 1 = 128` のときに `M^128` を `M^256` にしようとしてあふれます。
この二乗の結果は捨てられるので、検査の無い release ビルドでは答えに影響しません。

より大きな項が必要なら、`u128` を多倍長整数や剰余環に差し替えてください。
競技プログラミングでは答えを `10^9 + 7` などで割った余りにする問題が多く、
その場合は行列積の各要素を法の下で計算することになります。
